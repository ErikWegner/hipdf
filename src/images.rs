//! Image handling module for PDF documents
//!
//! This module provides functionality to embed various image formats (PNG, JPEG)
//! into PDF documents, with proper handling of transparency for PNG images.

use lopdf::content::Operation;
use lopdf::{dictionary, Dictionary, Document, Object, ObjectId, Stream};
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::Path;

use jpeg_decoder;
use miniz_oxide::deflate;
use png;

/// Supported image formats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageFormat {
    PNG,
    JPEG,
}

/// Image color space
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorSpace {
    DeviceRGB,
    DeviceGray,
    DeviceCMYK,
}

impl ColorSpace {
    /// Converts to PDF name object
    pub fn to_name(&self) -> Vec<u8> {
        match self {
            ColorSpace::DeviceRGB => b"DeviceRGB".to_vec(),
            ColorSpace::DeviceGray => b"DeviceGray".to_vec(),
            ColorSpace::DeviceCMYK => b"DeviceCMYK".to_vec(),
        }
    }

    /// Gets the number of components for this color space
    pub fn components(&self) -> u8 {
        match self {
            ColorSpace::DeviceGray => 1,
            ColorSpace::DeviceRGB => 3,
            ColorSpace::DeviceCMYK => 4,
        }
    }
}

/// Metadata about an image
#[derive(Debug, Clone)]
pub struct ImageMetadata {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Bits per component (usually 8)
    pub bits_per_component: u8,
    /// Color space
    pub color_space: ColorSpace,
    /// Whether the image has an alpha channel
    pub has_alpha: bool,
    /// Image format
    pub format: ImageFormat,
}

/// Represents an image that can be embedded in a PDF
#[derive(Debug, Clone)]
pub struct Image {
    /// Image metadata
    pub metadata: ImageMetadata,
    /// Raw image data (for JPEG, this is the file data; for PNG, this is decoded)
    pub data: Vec<u8>,
    /// Alpha channel data (for PNG with transparency)
    pub alpha_data: Option<Vec<u8>>,
    /// Original file path (if loaded from file)
    pub source_path: Option<String>,
}

impl Image {
    /// Loads an image from a file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let format = Self::detect_format(&buffer)?;
        let source_path = Some(path.to_string_lossy().to_string());

        match format {
            ImageFormat::PNG => Self::from_png_data(buffer, source_path),
            ImageFormat::JPEG => Self::from_jpeg_data(buffer, source_path),
        }
    }

    /// Creates an image from PNG data
    pub fn from_png_data(
        data: Vec<u8>,
        source_path: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let source = Cursor::new(data);
        let decoder = png::Decoder::new(source);
        let mut reader = decoder.read_info()?;
        
        // Get image info
        let info = reader.info();
        let width = info.width;
        let height = info.height;
        let color_type = info.color_type;
        
        // Calculate buffer size based on color type
        let bytes_per_pixel = match color_type {
            png::ColorType::Rgb => 3,
            png::ColorType::Rgba => 4,
            png::ColorType::Grayscale => 1,
            png::ColorType::GrayscaleAlpha => 2,
            png::ColorType::Indexed => {
                return Err("Indexed PNG not yet supported".into());
            }
        };
        
        let expected_len = (width as usize) * (height as usize) * bytes_per_pixel;
        let mut img_data = vec![0u8; expected_len];
        
        // Read the image data
        reader.next_frame(&mut img_data)?;
        
        // Determine color space and handle alpha
        let (color_space, has_alpha, processed_data, alpha_data) = match color_type {
            png::ColorType::Rgb => (ColorSpace::DeviceRGB, false, img_data, None),
            png::ColorType::Rgba => {
                // Separate RGB and Alpha channels
                let mut rgb_data = Vec::with_capacity((img_data.len() * 3) / 4);
                let mut alpha = Vec::with_capacity(img_data.len() / 4);
                
                for chunk in img_data.chunks_exact(4) {
                    rgb_data.push(chunk[0]);
                    rgb_data.push(chunk[1]);
                    rgb_data.push(chunk[2]);
                    alpha.push(chunk[3]);
                }
                
                (ColorSpace::DeviceRGB, true, rgb_data, Some(alpha))
            }
            png::ColorType::Grayscale => (ColorSpace::DeviceGray, false, img_data, None),
            png::ColorType::GrayscaleAlpha => {
                // Separate Gray and Alpha channels
                let mut gray_data = Vec::with_capacity(img_data.len() / 2);
                let mut alpha = Vec::with_capacity(img_data.len() / 2);
                
                for chunk in img_data.chunks_exact(2) {
                    gray_data.push(chunk[0]);
                    alpha.push(chunk[1]);
                }
                
                (ColorSpace::DeviceGray, true, gray_data, Some(alpha))
            }
            _ => return Err("Unsupported PNG color type".into()),
        };

        // PDF uses 8 bits per component for compatibility
        let bits_per_component = 8;

        let metadata = ImageMetadata {
            width,
            height,
            bits_per_component,
            color_space,
            has_alpha,
            format: ImageFormat::PNG,
        };

        Ok(Image {
            metadata,
            data: processed_data,
            alpha_data,
            source_path,
        })
    }

    /// Creates an image from JPEG data
    pub fn from_jpeg_data(
        data: Vec<u8>,
        source_path: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Use jpeg_decoder to get metadata
        let mut decoder = jpeg_decoder::Decoder::new(&data[..]);
        decoder.read_info()?;
        
        let info = decoder.info().ok_or("Failed to read JPEG info")?;
        
        let color_space = match info.pixel_format {
            jpeg_decoder::PixelFormat::RGB24 => ColorSpace::DeviceRGB,
            jpeg_decoder::PixelFormat::L8 => ColorSpace::DeviceGray,
            jpeg_decoder::PixelFormat::CMYK32 => ColorSpace::DeviceCMYK,
            _ => return Err("Unsupported JPEG pixel format".into()),
        };

        let metadata = ImageMetadata {
            width: info.width as u32,
            height: info.height as u32,
            bits_per_component: 8,
            color_space,
            has_alpha: false,
            format: ImageFormat::JPEG,
        };

        Ok(Image {
            metadata,
            data, // For JPEG, we keep the original compressed data
            alpha_data: None,
            source_path,
        })
    }

    /// Detects the format of image data
    fn detect_format(data: &[u8]) -> Result<ImageFormat, Box<dyn std::error::Error>> {
        if data.len() < 8 {
            return Err("Invalid image data".into());
        }

        // Check PNG signature
        if data[0..8] == [137, 80, 78, 71, 13, 10, 26, 10] {
            return Ok(ImageFormat::PNG);
        }

        // Check JPEG signature
        if data[0..2] == [0xFF, 0xD8] {
            return Ok(ImageFormat::JPEG);
        }

        Err("Unknown image format".into())
    }

    /// Gets the dimensions of the image
    pub fn dimensions(&self) -> (u32, u32) {
        (self.metadata.width, self.metadata.height)
    }

    /// Gets the aspect ratio (width / height)
    pub fn aspect_ratio(&self) -> f32 {
        self.metadata.width as f32 / self.metadata.height as f32
    }
}

/// Manager for embedding images in PDF documents
pub struct ImageManager {
    /// Cached images with their PDF object IDs
    images: Vec<(Image, ObjectId, Option<ObjectId>)>, // (image, image_id, mask_id)
    /// Counter for generating unique resource names
    name_counter: usize,
}

impl Default for ImageManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageManager {
    /// Creates a new image manager
    pub fn new() -> Self {
        ImageManager {
            images: Vec::new(),
            name_counter: 0,
        }
    }

    /// Embeds an image in the PDF document
    pub fn embed_image(
        &mut self,
        doc: &mut Document,
        image: Image,
    ) -> Result<ObjectId, Box<dyn std::error::Error>> {
        // Check if this image was already embedded
        for (cached_img, obj_id, _) in &self.images {
            if let (Some(path1), Some(path2)) = (&cached_img.source_path, &image.source_path) {
                if path1 == path2 {
                    return Ok(*obj_id);
                }
            }
        }

        let (image_id, mask_id) = match image.metadata.format {
            ImageFormat::JPEG => (self.embed_jpeg(doc, &image)?, None),
            ImageFormat::PNG => {
                let (img_id, mask) = self.embed_png(doc, &image)?;
                (img_id, mask)
            }
        };

        self.images.push((image, image_id, mask_id));
        Ok(image_id)
    }

    /// Embeds a JPEG image
    fn embed_jpeg(
        &self,
        doc: &mut Document,
        image: &Image,
    ) -> Result<ObjectId, Box<dyn std::error::Error>> {
        let dict = dictionary! {
            "Type" => "XObject",
            "Subtype" => "Image",
            "Width" => image.metadata.width as i32,
            "Height" => image.metadata.height as i32,
            "BitsPerComponent" => image.metadata.bits_per_component as i32,
            "ColorSpace" => Object::Name(image.metadata.color_space.to_name()),
            "Filter" => "DCTDecode",
        };

        let stream = Stream::new(dict, image.data.clone());
        Ok(doc.add_object(stream))
    }

    /// Embeds a PNG image
    fn embed_png(
        &self,
        doc: &mut Document,
        image: &Image,
    ) -> Result<(ObjectId, Option<ObjectId>), Box<dyn std::error::Error>> {
        // Create soft mask for alpha channel if present
        let mask_id = if let Some(ref alpha_data) = image.alpha_data {
            let mask_dict = dictionary! {
                "Type" => "XObject",
                "Subtype" => "Image",
                "Width" => image.metadata.width as i32,
                "Height" => image.metadata.height as i32,
                "BitsPerComponent" => 8,
                "ColorSpace" => "DeviceGray",
                "Filter" => "FlateDecode",
                "Decode" => vec![0.into(), 1.into()],
            };

            // Compress alpha data with zlib
            let compressed = deflate::compress_to_vec_zlib(alpha_data, 6);
            let mask_stream = Stream::new(mask_dict, compressed);
            Some(doc.add_object(mask_stream))
        } else {
            None
        };

        // Create main image
        let mut dict = dictionary! {
            "Type" => "XObject",
            "Subtype" => "Image",
            "Width" => image.metadata.width as i32,
            "Height" => image.metadata.height as i32,
            "BitsPerComponent" => image.metadata.bits_per_component as i32,
            "ColorSpace" => Object::Name(image.metadata.color_space.to_name()),
            "Filter" => "FlateDecode",
        };

        // Add decode array for proper color interpretation
        match image.metadata.color_space {
            ColorSpace::DeviceRGB => {
                dict.set("Decode", vec![0.into(), 1.into(), 0.into(), 1.into(), 0.into(), 1.into()]);
            }
            ColorSpace::DeviceGray => {
                dict.set("Decode", vec![0.into(), 1.into()]);
            }
            _ => {}
        }

        // Add soft mask reference if we have alpha
        if let Some(mask_id) = mask_id {
            dict.set("SMask", Object::Reference(mask_id));
        }

        // Compress image data with zlib (not just deflate)
        let compressed = deflate::compress_to_vec_zlib(&image.data, 6);
        let stream = Stream::new(dict, compressed);
        let image_id = doc.add_object(stream);

        Ok((image_id, mask_id))
    }

    /// Adds an image to page resources and returns its resource name
    pub fn add_to_resources(
        &mut self,
        resources: &mut Dictionary,
        image_id: ObjectId,
    ) -> String {
        let name = format!("Im{}", self.name_counter);
        self.name_counter += 1;

        // Get or create XObject dictionary
        let xobject = if let Ok(Object::Dictionary(dict)) = resources.get(b"XObject") {
            dict.clone()
        } else {
            Dictionary::new()
        };

        let mut xobject = xobject;
        xobject.set(name.clone(), Object::Reference(image_id));
        resources.set("XObject", xobject);

        name
    }

    /// Creates a Do operation to draw an image
    pub fn create_draw_operation(resource_name: &str) -> Operation {
        Operation::new("Do", vec![Object::Name(resource_name.as_bytes().to_vec())])
    }

    /// Creates operations to draw an image at a specific position and size
    pub fn draw_image(
        resource_name: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Vec<Operation> {
        vec![
            Operation::new("q", vec![]),
            Operation::new(
                "cm",
                vec![
                    width.into(),
                    0.0.into(),
                    0.0.into(),
                    height.into(),
                    x.into(),
                    y.into(),
                ],
            ),
            Self::create_draw_operation(resource_name),
            Operation::new("Q", vec![]),
        ]
    }

    /// Creates operations to draw an image with rotation
    pub fn draw_image_rotated(
        resource_name: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        angle_degrees: f32,
    ) -> Vec<Operation> {
        let angle_rad = angle_degrees * std::f32::consts::PI / 180.0;
        let cos = angle_rad.cos();
        let sin = angle_rad.sin();

        // Calculate center point for rotation
        let cx = x + width / 2.0;
        let cy = y + height / 2.0;

        vec![
            Operation::new("q", vec![]),
            // Translate to center
            Operation::new("cm", vec![
                1.0.into(), 0.0.into(), 0.0.into(), 1.0.into(),
                cx.into(), cy.into(),
            ]),
            // Rotate
            Operation::new("cm", vec![
                cos.into(), sin.into(), (-sin).into(), cos.into(),
                0.0.into(), 0.0.into(),
            ]),
            // Scale and translate back
            Operation::new("cm", vec![
                width.into(), 0.0.into(), 0.0.into(), height.into(),
                (-width / 2.0).into(), (-height / 2.0).into(),
            ]),
            Self::create_draw_operation(resource_name),
            Operation::new("Q", vec![]),
        ]
    }

    /// Creates operations to draw an image maintaining aspect ratio
    pub fn draw_image_fit(
        resource_name: &str,
        image: &Image,
        x: f32,
        y: f32,
        max_width: f32,
        max_height: f32,
    ) -> Vec<Operation> {
        let aspect = image.aspect_ratio();
        
        let (width, height) = if max_width / max_height > aspect {
            // Height is the limiting factor
            (max_height * aspect, max_height)
        } else {
            // Width is the limiting factor
            (max_width, max_width / aspect)
        };

        // Center the image in the box
        let offset_x = (max_width - width) / 2.0;
        let offset_y = (max_height - height) / 2.0;

        Self::draw_image(resource_name, x + offset_x, y + offset_y, width, height)
    }

    /// Gets the number of embedded images
    pub fn count(&self) -> usize {
        self.images.len()
    }

    /// Gets an embedded image by index
    pub fn get(&self, index: usize) -> Option<&(Image, ObjectId, Option<ObjectId>)> {
        self.images.get(index)
    }

    /// Clears all cached images
    pub fn clear(&mut self) {
        self.images.clear();
        self.name_counter = 0;
    }
}

/// Helper struct for building image operations
pub struct ImageBuilder {
    operations: Vec<Operation>,
}

impl Default for ImageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageBuilder {
    /// Creates a new image builder
    pub fn new() -> Self {
        ImageBuilder {
            operations: Vec::new(),
        }
    }

    /// Adds an image at a specific position
    pub fn add_image(
        mut self,
        resource_name: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Self {
        self.operations.extend(ImageManager::draw_image(
            resource_name,
            x,
            y,
            width,
            height,
        ));
        self
    }

    /// Adds a rotated image
    pub fn add_image_rotated(
        mut self,
        resource_name: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        angle: f32,
    ) -> Self {
        self.operations.extend(ImageManager::draw_image_rotated(
            resource_name,
            x,
            y,
            width,
            height,
            angle,
        ));
        self
    }

    /// Adds an image with aspect ratio preservation
    pub fn add_image_fit(
        mut self,
        resource_name: &str,
        image: &Image,
        x: f32,
        y: f32,
        max_width: f32,
        max_height: f32,
    ) -> Self {
        self.operations.extend(ImageManager::draw_image_fit(
            resource_name,
            image,
            x,
            y,
            max_width,
            max_height,
        ));
        self
    }

    /// Adds a custom operation
    pub fn add_operation(mut self, op: Operation) -> Self {
        self.operations.push(op);
        self
    }

    /// Builds the operations
    pub fn build(self) -> Vec<Operation> {
        self.operations
    }
}

/// Convenience functions for common image operations
pub mod utils {
    use super::*;

    /// Creates a thumbnail grid of images
    pub fn create_thumbnail_grid(
        images: &[(String, &Image)], // (resource_name, image)
        x: f32,
        y: f32,
        cols: usize,
        thumb_size: f32,
        spacing: f32,
    ) -> Vec<Operation> {
        let mut operations = Vec::new();

        for (index, (resource_name, image)) in images.iter().enumerate() {
            let col = index % cols;
            let row = index / cols;

            let img_x = x + (col as f32) * (thumb_size + spacing);
            let img_y = y - (row as f32) * (thumb_size + spacing);

            operations.extend(ImageManager::draw_image_fit(
                resource_name,
                image,
                img_x,
                img_y,
                thumb_size,
                thumb_size,
            ));
        }

        operations
    }

    /// Creates a watermark from an image
    pub fn create_watermark(
        resource_name: &str,
        page_width: f32,
        page_height: f32,
        opacity: f32,
    ) -> Vec<Operation> {
        let mut operations = Vec::new();

        // Save graphics state
        operations.push(Operation::new("q", vec![]));

        // Set opacity (requires ExtGState)
        if opacity < 1.0 {
            // This is a placeholder - actual implementation needs ExtGState
            operations.push(Operation::new(
                "%",
                vec![Object::string_literal(format!("Opacity: {}", opacity))],
            ));
        }

        // Draw centered watermark
        let size = page_width.min(page_height) * 0.5;
        let x = (page_width - size) / 2.0;
        let y = (page_height - size) / 2.0;

        operations.extend(ImageManager::draw_image(resource_name, x, y, size, size));

        // Restore graphics state
        operations.push(Operation::new("Q", vec![]));

        operations
    }
}

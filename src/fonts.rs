//! Font handling module for PDF documents with standard and custom font support
//!
//! This module provides functionality to use fonts in PDF documents including:
//! - All 14 standard PDF fonts (no embedding required)
//! - TrueType/OpenType font embedding with Unicode support
//! - Font subsetting for smaller file sizes
//! - Text encoding and measurement
//! - Advanced text operations (positioning, styling, transformations)

use lopdf::content::Operation;
use lopdf::{dictionary, Dictionary, Document, Object, ObjectId, Stream};
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use ttf_parser::{Face, GlyphId};

/// Standard PDF fonts that don't require embedding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StandardFont {
    TimesRoman,
    TimesBold,
    TimesItalic,
    TimesBoldItalic,
    Helvetica,
    HelveticaBold,
    HelveticaOblique,
    HelveticaBoldOblique,
    Courier,
    CourierBold,
    CourierOblique,
    CourierBoldOblique,
    Symbol,
    ZapfDingbats,
}

impl StandardFont {
    /// Gets the PostScript name for this standard font
    pub fn postscript_name(&self) -> &'static str {
        match self {
            StandardFont::TimesRoman => "Times-Roman",
            StandardFont::TimesBold => "Times-Bold",
            StandardFont::TimesItalic => "Times-Italic",
            StandardFont::TimesBoldItalic => "Times-BoldItalic",
            StandardFont::Helvetica => "Helvetica",
            StandardFont::HelveticaBold => "Helvetica-Bold",
            StandardFont::HelveticaOblique => "Helvetica-Oblique",
            StandardFont::HelveticaBoldOblique => "Helvetica-BoldOblique",
            StandardFont::Courier => "Courier",
            StandardFont::CourierBold => "Courier-Bold",
            StandardFont::CourierOblique => "Courier-Oblique",
            StandardFont::CourierBoldOblique => "Courier-BoldOblique",
            StandardFont::Symbol => "Symbol",
            StandardFont::ZapfDingbats => "ZapfDingbats",
        }
    }

    /// Gets the font family name
    pub fn family(&self) -> &'static str {
        match self {
            StandardFont::TimesRoman
            | StandardFont::TimesBold
            | StandardFont::TimesItalic
            | StandardFont::TimesBoldItalic => "Times",
            StandardFont::Helvetica
            | StandardFont::HelveticaBold
            | StandardFont::HelveticaOblique
            | StandardFont::HelveticaBoldOblique => "Helvetica",
            StandardFont::Courier
            | StandardFont::CourierBold
            | StandardFont::CourierOblique
            | StandardFont::CourierBoldOblique => "Courier",
            StandardFont::Symbol => "Symbol",
            StandardFont::ZapfDingbats => "ZapfDingbats",
        }
    }

    /// Checks if this font is bold
    pub fn is_bold(&self) -> bool {
        matches!(
            self,
            StandardFont::TimesBold
                | StandardFont::TimesBoldItalic
                | StandardFont::HelveticaBold
                | StandardFont::HelveticaBoldOblique
                | StandardFont::CourierBold
                | StandardFont::CourierBoldOblique
        )
    }

    /// Checks if this font is italic/oblique
    pub fn is_italic(&self) -> bool {
        matches!(
            self,
            StandardFont::TimesItalic
                | StandardFont::TimesBoldItalic
                | StandardFont::HelveticaOblique
                | StandardFont::HelveticaBoldOblique
                | StandardFont::CourierOblique
                | StandardFont::CourierBoldOblique
        )
    }
}

/// Font type classification
#[derive(Debug, Clone, PartialEq)]
pub enum FontType {
    /// Standard PDF font (14 built-in fonts)
    Standard(StandardFont),
    /// TrueType font with embedded data
    TrueType { data: Vec<u8> },
    /// OpenType font with embedded data
    OpenType { data: Vec<u8> },
}

/// Font metadata
#[derive(Debug, Clone)]
pub struct FontMetadata {
    /// Font family name
    pub family: String,
    /// PostScript name used in PDF dictionaries
    pub postscript_name: String,
    /// Font weight (100-900)
    pub weight: u16,
    /// Whether the font is italic
    pub italic: bool,
    /// Font encoding
    pub encoding: String,
    /// Whether the font supports Unicode
    pub unicode: bool,
}

/// Detailed metrics derived from the font file (scaled to PDF units)
#[derive(Debug, Clone)]
pub struct FontMetrics {
    /// Units per em reported by the font
    pub units_per_em: u16,
    /// Ascender height (scaled to 1000 units)
    pub ascender: i32,
    /// Descender depth (scaled to 1000 units)
    pub descender: i32,
    /// Cap height (scaled to 1000 units)
    pub cap_height: i32,
    /// Italic angle in degrees
    pub italic_angle: f32,
    /// Font bounding box (xmin, ymin, xmax, ymax) scaled to 1000 units
    pub bbox: [i32; 4],
    /// Default glyph width used for DW entry (scaled to 1000 units)
    pub default_width: u16,
}

/// Represents a font that can be used in a PDF
#[derive(Debug, Clone)]
pub struct Font {
    /// Font type and data
    pub font_type: FontType,
    /// Font metadata
    pub metadata: FontMetadata,
    /// Source file path (if loaded from file)
    pub source_path: Option<String>,
    /// Mapping from Unicode codepoints to glyph IDs
    pub glyph_mapping: HashMap<u32, u16>,
    /// Glyph width table (glyph ID -> width in 1/1000 text space)
    pub glyph_widths: HashMap<u16, u16>,
    /// Glyph ID to use when no mapping is found
    pub missing_glyph_id: u16,
    /// Optional metrics gathered from the font file
    pub metrics: Option<FontMetrics>,
}

#[derive(Debug)]
struct ParsedFontGeometry {
    glyph_mapping: HashMap<u32, u16>,
    glyph_widths: HashMap<u16, u16>,
    missing_glyph_id: u16,
    metrics: FontMetrics,
}

impl Font {
    /// Creates a font from a standard PDF font
    pub fn standard(font: StandardFont) -> Self {
        let metadata = FontMetadata {
            family: font.family().to_string(),
            postscript_name: font.postscript_name().to_string(),
            weight: if font.is_bold() { 700 } else { 400 },
            italic: font.is_italic(),
            encoding: "WinAnsiEncoding".to_string(),
            unicode: false,
        };

        Font {
            font_type: FontType::Standard(font),
            metadata,
            source_path: None,
            glyph_mapping: HashMap::new(),
            glyph_widths: HashMap::new(),
            missing_glyph_id: 0,
            metrics: None,
        }
    }

    /// Loads a TrueType font from a file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let source_path = Some(path.to_string_lossy().to_string());
        Self::from_bytes(data, source_path)
    }

    /// Creates a font from TrueType/OpenType bytes
    pub fn from_bytes(
        data: Vec<u8>,
        source_path: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Simple format detection
        let font_type = if data.len() >= 4 {
            let tag = &data[0..4];
            if tag == b"\x00\x01\x00\x00" || tag == b"true" {
                FontType::TrueType { data: data.clone() }
            } else if tag == b"OTTO" {
                FontType::OpenType { data: data.clone() }
            } else {
                return Err("Unknown font format".into());
            }
        } else {
            return Err("Invalid font data".into());
        };

        let face = Face::parse(&data, 0).map_err(|e| format!("Failed to parse font: {:?}", e))?;

        let geometry = Self::extract_font_geometry(&face)?;
        let metadata = Self::extract_metadata_from_face(&face)?;

        Ok(Font {
            font_type,
            metadata,
            source_path,
            glyph_mapping: geometry.glyph_mapping,
            glyph_widths: geometry.glyph_widths,
            missing_glyph_id: geometry.missing_glyph_id,
            metrics: Some(geometry.metrics),
        })
    }

    fn extract_font_geometry(
        face: &Face,
    ) -> Result<ParsedFontGeometry, Box<dyn std::error::Error>> {
        let mut units_per_em = face.units_per_em();
        if units_per_em == 0 {
            units_per_em = 1000;
        }
        let mut glyph_mapping = HashMap::new();

        if let Some(cmap) = face.tables().cmap {
            for subtable in cmap.subtables {
                if !subtable.is_unicode() {
                    continue;
                }

                subtable.codepoints(|codepoint| {
                    if let Some(glyph) = subtable.glyph_index(codepoint) {
                        glyph_mapping.entry(codepoint).or_insert(glyph.0);
                    }
                });
            }
        }

        if glyph_mapping.is_empty() {
            return Err("Font cmap table did not produce any glyph mappings".into());
        }

        let mut glyph_widths = HashMap::new();
        for glyph_id in 0..face.number_of_glyphs() {
            let gid = GlyphId(glyph_id);
            if let Some(advance) = face.glyph_hor_advance(gid) {
                let width = Self::scale_width_to_pdf_units(advance, units_per_em);
                glyph_widths.insert(glyph_id, width);
            }
        }

        let missing_glyph_id = face.glyph_index('?').map(|g| g.0).unwrap_or(0);

        let italic_angle = face.italic_angle().unwrap_or(0.0);
        let ascender = Self::scale_metric_to_pdf_units(face.ascender(), units_per_em);
        let descender = Self::scale_metric_to_pdf_units(face.descender(), units_per_em);
        let cap_height = ascender;

        let rect = face.global_bounding_box();
        let bbox = [
            Self::scale_metric_to_pdf_units(rect.x_min, units_per_em),
            Self::scale_metric_to_pdf_units(rect.y_min, units_per_em),
            Self::scale_metric_to_pdf_units(rect.x_max, units_per_em),
            Self::scale_metric_to_pdf_units(rect.y_max, units_per_em),
        ];

        let default_width = {
            let space_gid = glyph_mapping
                .get(&(b' ' as u32))
                .copied()
                .unwrap_or(missing_glyph_id);
            glyph_widths
                .get(&space_gid)
                .copied()
                .or_else(|| {
                    if glyph_widths.is_empty() {
                        None
                    } else {
                        let sum: u32 = glyph_widths.values().map(|&w| w as u32).sum();
                        Some((sum / glyph_widths.len() as u32) as u16)
                    }
                })
                .unwrap_or(1000)
        };

        let metrics = FontMetrics {
            units_per_em,
            ascender,
            descender,
            cap_height,
            italic_angle,
            bbox,
            default_width,
        };

        glyph_widths
            .entry(missing_glyph_id)
            .or_insert(default_width);

        Ok(ParsedFontGeometry {
            glyph_mapping,
            glyph_widths,
            missing_glyph_id,
            metrics,
        })
    }

    fn scale_width_to_pdf_units(width: u16, units_per_em: u16) -> u16 {
        if units_per_em == 0 {
            return width;
        }
        let scaled = (width as f32 * 1000.0) / units_per_em as f32;
        scaled.round().clamp(0.0, 65535.0) as u16
    }

    fn scale_metric_to_pdf_units(value: i16, units_per_em: u16) -> i32 {
        if units_per_em == 0 {
            return value as i32;
        }
        let scaled = (value as f32 * 1000.0) / units_per_em as f32;
        scaled.round() as i32
    }

    /// Extracts metadata directly from ttf_parser Face (simpler & more reliable)
    fn extract_metadata_from_face(face: &Face) -> Result<FontMetadata, Box<dyn std::error::Error>> {
        // Get family name - ttf_parser handles all the complexity
        let family = face
            .names()
            .into_iter()
            .filter(|name| name.name_id == 16 || name.name_id == 1) // Typographic family or Font family
            .filter(|name| name.is_unicode())
            .find_map(|name| name.to_string())
            .unwrap_or_else(|| "Unknown Font".to_string());

        // Get PostScript name
        let raw_postscript = face
            .names()
            .into_iter()
            .filter(|name| name.name_id == 6) // PostScript name
            .filter(|name| name.is_unicode())
            .find_map(|name| name.to_string());

        let postscript_name = raw_postscript
            .filter(|name| !name.is_empty())
            .map(|name| Self::sanitize_postscript_name(&name))
            .unwrap_or_else(|| Self::sanitize_postscript_name(&family));

        // Get weight from OS/2 table
        let weight = face.weight().to_number();

        // Check if italic
        let italic = face.is_italic();

        Ok(FontMetadata {
            family,
            postscript_name,
            weight,
            italic,
            encoding: "Identity-H".to_string(),
            unicode: true,
        })
    }

    fn sanitize_postscript_name(name: &str) -> String {
        let mut sanitized = String::with_capacity(name.len());
        let mut last_was_hyphen = false;

        for ch in name.chars() {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '+') {
                sanitized.push(ch);
                last_was_hyphen = false;
            } else if ch.is_whitespace() && !sanitized.is_empty() && !last_was_hyphen {
                sanitized.push('-');
                last_was_hyphen = true;
            }
        }

        let sanitized = sanitized.trim_matches('-').to_string();
        let mut sanitized = if sanitized.is_empty() {
            "EmbeddedFont".to_string()
        } else {
            sanitized
        };

        if sanitized.len() > 127 {
            sanitized.truncate(127);
        }

        sanitized
    }

    /// Gets the font's family name
    pub fn family(&self) -> &str {
        &self.metadata.family
    }

    /// Checks if the font is bold
    pub fn is_bold(&self) -> bool {
        self.metadata.weight >= 600
    }

    /// Checks if the font is italic
    pub fn is_italic(&self) -> bool {
        self.metadata.italic
    }

    /// Estimates text width using available font metrics
    pub fn text_width(&self, text: &str, size: f32) -> f32 {
        match &self.font_type {
            FontType::Standard(_) => text.len() as f32 * size * 0.5,
            FontType::TrueType { .. } | FontType::OpenType { .. } => {
                let default_width = self
                    .metrics
                    .as_ref()
                    .map(|m| m.default_width)
                    .unwrap_or(1000);

                let total_width: u32 = text
                    .chars()
                    .map(|ch| {
                        let glyph_id = self
                            .glyph_mapping
                            .get(&(ch as u32))
                            .copied()
                            .unwrap_or(self.missing_glyph_id);
                        self.glyph_widths
                            .get(&glyph_id)
                            .copied()
                            .unwrap_or(default_width) as u32
                    })
                    .sum();

                (total_width as f32 * size) / 1000.0
            }
        }
    }

    /// Measures the width of a single character
    pub fn char_width(&self, ch: char, size: f32) -> f32 {
        match &self.font_type {
            FontType::Standard(_) => size * 0.5,
            FontType::TrueType { .. } | FontType::OpenType { .. } => {
                let default_width = self
                    .metrics
                    .as_ref()
                    .map(|m| m.default_width)
                    .unwrap_or(1000);

                let glyph_id = self
                    .glyph_mapping
                    .get(&(ch as u32))
                    .copied()
                    .unwrap_or(self.missing_glyph_id);
                
                let width = self
                    .glyph_widths
                    .get(&glyph_id)
                    .copied()
                    .unwrap_or(default_width);

                (width as f32 * size) / 1000.0
            }
        }
    }

    /// Encodes text for use with this font in PDF
    pub fn encode_text(&self, text: &str) -> Vec<u8> {
        match &self.font_type {
            FontType::Standard(_) => {
                // Standard fonts use simple string encoding
                text.as_bytes().to_vec()
            }
            FontType::TrueType { .. } | FontType::OpenType { .. } => {
                // Type0 fonts expect glyph IDs encoded as big-endian values
                let mut encoded = Vec::with_capacity(text.len() * 2);
                for ch in text.chars() {
                    let codepoint = ch as u32;
                    let glyph_id = self
                        .glyph_mapping
                        .get(&codepoint)
                        .copied()
                        .unwrap_or(self.missing_glyph_id);

                    encoded.push((glyph_id >> 8) as u8);
                    encoded.push((glyph_id & 0xFF) as u8);
                }
                encoded
            }
        }
    }

    /// Returns true if this font requires composite glyph encoding
    pub fn needs_utf16_encoding(&self) -> bool {
        matches!(
            self.font_type,
            FontType::TrueType { .. } | FontType::OpenType { .. }
        )
    }
}

/// Manager for fonts in PDF documents
pub struct FontManager {
    /// Cached fonts with their PDF object IDs
    fonts: Vec<(Font, ObjectId, String)>, // (font, font_id, resource_name)
    /// Counter for generating unique resource names
    name_counter: usize,
    /// Standard fonts cache
    standard_fonts: HashMap<StandardFont, ObjectId>,
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FontManager {
    /// Creates a new font manager
    pub fn new() -> Self {
        FontManager {
            fonts: Vec::new(),
            name_counter: 0,
            standard_fonts: HashMap::new(),
        }
    }

    /// Embeds a font in the PDF document
    pub fn embed_font(
        &mut self,
        doc: &mut Document,
        font: Font,
    ) -> Result<(ObjectId, String), Box<dyn std::error::Error>> {
        // Check if this font was already embedded
        for (cached_font, obj_id, resource_name) in &self.fonts {
            if let (Some(path1), Some(path2)) = (&cached_font.source_path, &font.source_path) {
                if path1 == path2 {
                    return Ok((*obj_id, resource_name.clone()));
                }
            }
        }

        let font_id = match &font.font_type {
            FontType::Standard(std_font) => {
                if let Some(&cached_id) = self.standard_fonts.get(std_font) {
                    cached_id
                } else {
                    let id = self.embed_standard_font(doc, *std_font)?;
                    self.standard_fonts.insert(*std_font, id);
                    id
                }
            }
            FontType::TrueType { .. } | FontType::OpenType { .. } => {
                self.embed_truetype_font(doc, &font)?
            }
        };

        let resource_name = format!("F{}", self.name_counter);
        self.name_counter += 1;

        self.fonts.push((font, font_id, resource_name.clone()));
        Ok((font_id, resource_name))
    }

    /// Embeds a standard PDF font
    fn embed_standard_font(
        &self,
        doc: &mut Document,
        font: StandardFont,
    ) -> Result<ObjectId, Box<dyn std::error::Error>> {
        let font_dict = dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => Object::Name(font.postscript_name().as_bytes().to_vec()),
            "Encoding" => "WinAnsiEncoding",
        };

        Ok(doc.add_object(font_dict))
    }

    /// Embeds a TrueType/OpenType font with Unicode support
    fn embed_truetype_font(
        &self,
        doc: &mut Document,
        font: &Font,
    ) -> Result<ObjectId, Box<dyn std::error::Error>> {
        let font_data = match &font.font_type {
            FontType::TrueType { data } | FontType::OpenType { data } => data,
            _ => return Err("Not a TrueType/OpenType font".into()),
        };

        // For maximum compatibility, we must provide an explicit CIDToGIDMap stream instead of
        // relying on the "/Identity" shortcut, which is poorly supported by some viewers
        // (including macOS Preview) and can cause character encoding issues.

        // We need to parse the font here to get the total number of glyphs.
        let face = ttf_parser::Face::parse(font_data, 0)
            .map_err(|e| format!("Failed to parse font for GID map: {:?}", e))?;
        let num_glyphs = face.number_of_glyphs();

        // Create the identity mapping data: CID `i` maps to GID `i`.
        // This is a stream of big-endian u16 values.
        let mut gid_map_data = Vec::with_capacity(num_glyphs as usize * 2);
        for i in 0..num_glyphs {
            gid_map_data.push((i >> 8) as u8); // High byte of GID
            gid_map_data.push((i & 0xFF) as u8); // Low byte of GID
        }

        // Create a stream object for the mapping and add it to the document.
        let cid_to_gid_map_stream = Stream::new(dictionary!{}, gid_map_data);
        let cid_to_gid_map_id = doc.add_object(cid_to_gid_map_stream);

        // Create font file stream
        let font_file = Stream::new(
            dictionary! {
                "Length1" => font_data.len() as i64,
            },
            font_data.clone(),
        );
        let font_file_id = doc.add_object(font_file);

        let metrics = font.metrics.as_ref();
        let bbox = metrics.map(|m| m.bbox).unwrap_or([-500, -200, 1000, 900]);
        let ascender = metrics.map(|m| m.ascender).unwrap_or(900);
        let descender = metrics.map(|m| m.descender).unwrap_or(-200);
        let cap_height = metrics.map(|m| m.cap_height).unwrap_or(700);
        let italic_angle = metrics.map(|m| m.italic_angle as f64).unwrap_or_else(|| {
            if font.metadata.italic {
                -12.0
            } else {
                0.0
            }
        });
        let default_width = metrics.map(|m| m.default_width as i64).unwrap_or(1000);

        // Create font descriptor
        let font_descriptor = dictionary! {
            "Type" => "FontDescriptor",
            "FontName" => Object::Name(font.metadata.postscript_name.as_bytes().to_vec()),
            "FontFamily" => Object::string_literal(font.metadata.family.as_str()),
            "Flags" => 32, // Non-symbolic font
            "FontBBox" => vec![
                bbox[0].into(),
                bbox[1].into(),
                bbox[2].into(),
                bbox[3].into(),
            ],
            "ItalicAngle" => italic_angle,
            "Ascent" => ascender,
            "Descent" => descender,
            "CapHeight" => cap_height,
            "StemV" => if font.is_bold() { 120 } else { 80 },
            "FontFile2" => Object::Reference(font_file_id),
        };
        let descriptor_id = doc.add_object(font_descriptor);

        // Create CIDFont for Unicode support
        let mut cid_font = dictionary! {
            "Type" => "Font",
            "Subtype" => "CIDFontType2",
            "BaseFont" => Object::Name(font.metadata.postscript_name.as_bytes().to_vec()),
            "CIDSystemInfo" => dictionary! {
                "Registry" => Object::string_literal("Adobe"),
                "Ordering" => Object::string_literal("Identity"),
                "Supplement" => 0,
            },
            "FontDescriptor" => Object::Reference(descriptor_id),
            "DW" => default_width,
            // --- USE THE EXPLICIT STREAM REFERENCE ---
            "CIDToGIDMap" => Object::Reference(cid_to_gid_map_id),
        };
        if let Some(width_array) = Self::build_width_array(&font.glyph_widths) {
            cid_font.set("W", width_array);
        }
        let cid_font_id = doc.add_object(cid_font);

        // Create ToUnicode CMap for text extraction
        let to_unicode = Self::create_to_unicode_cmap(font);
        let to_unicode_id = doc.add_object(to_unicode);

        // Create Type0 font (composite font)
        let type0_font = dictionary! {
            "Type" => "Font",
            "Subtype" => "Type0",
            "BaseFont" => Object::Name(font.metadata.postscript_name.as_bytes().to_vec()),
            "Encoding" => "Identity-H",
            "DescendantFonts" => vec![Object::Reference(cid_font_id)],
            "ToUnicode" => Object::Reference(to_unicode_id),
        };

        Ok(doc.add_object(type0_font))
    }

    /// Builds the /W (widths) array for a CIDFont.
    /// The format is a series of entries, where each entry can be:
    /// - c [w1 w2 ... wn]  (for n consecutive CIDs starting at c)
    /// - c_first c_last w  (for a range of CIDs with the same width)
    /// This implementation uses the first format for simplicity and correctness.
    fn build_width_array(widths: &HashMap<u16, u16>) -> Option<Object> {
        if widths.is_empty() {
            return None;
        }

        let mut glyph_ids: Vec<u16> = widths.keys().copied().collect();
        glyph_ids.sort_unstable();

        let mut entries: Vec<Object> = Vec::new();
        if glyph_ids.is_empty() {
            return Some(Object::Array(entries));
        }

        let mut iter = glyph_ids.into_iter();

        // Start the first range of consecutive glyph IDs
        let mut start_glyph = iter.next().unwrap();
        let mut last_glyph = start_glyph;
        let mut range_widths = vec![Object::Integer(
            widths.get(&start_glyph).copied().unwrap_or(1000) as i64,
        )];

        for glyph_id in iter {
            if glyph_id == last_glyph + 1 {
                // This glyph is consecutive, add its width to the current range
                range_widths.push(Object::Integer(
                    widths.get(&glyph_id).copied().unwrap_or(1000) as i64,
                ));
                last_glyph = glyph_id;
            } else {
                // A gap was found, ending the current range.
                // Add the completed range to our list of entries.
                entries.push(Object::Integer(start_glyph as i64));
                entries.push(Object::Array(range_widths));

                // Start a new range with the current glyph
                start_glyph = glyph_id;
                last_glyph = glyph_id;
                range_widths = vec![Object::Integer(
                    widths.get(&glyph_id).copied().unwrap_or(1000) as i64,
                )];
            }
        }

        // Add the very last processed range to the entries
        if !range_widths.is_empty() {
            entries.push(Object::Integer(start_glyph as i64));
            entries.push(Object::Array(range_widths));
        }

        Some(Object::Array(entries))
    }

    /// Creates a ToUnicode CMap for text extraction based on glyph mappings
    fn create_to_unicode_cmap(font: &Font) -> Stream {
        let mut glyph_to_unicode: BTreeMap<u16, u32> = BTreeMap::new();
        for (codepoint, glyph) in &font.glyph_mapping {
            glyph_to_unicode.entry(*glyph).or_insert(*codepoint);
        }

        let mut cmap = String::from("/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n/CIDSystemInfo\n<< /Registry (Adobe)\n/Ordering (UCS)\n/Supplement 0\n>> def\n/CMapName /Adobe-Identity-UCS def\n/CMapType 2 def\n1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n");

        let entries: Vec<(u16, u32)> = glyph_to_unicode.into_iter().collect();
        for chunk in entries.chunks(100) {
            cmap.push_str(&format!("{} beginbfchar\n", chunk.len()));
            for (glyph_id, codepoint) in chunk {
                if let Some(ch) = char::from_u32(*codepoint) {
                    let mut buffer = [0u16; 2];
                    let encoded = ch.encode_utf16(&mut buffer);
                    let mut unicode_hex = String::new();
                    for unit in encoded {
                        unicode_hex.push_str(&format!("{:04X}", unit));
                    }
                    cmap.push_str(&format!("<{:04X}> <{}>\n", glyph_id, unicode_hex));
                }
            }
            cmap.push_str("endbfchar\n");
        }

        cmap.push_str("endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend");

        Stream::new(dictionary! {}, cmap.into_bytes())
    }

    /// Adds a font to page resources
    pub fn add_to_resources(
        &self,
        resources: &mut Dictionary,
        font_id: ObjectId,
        resource_name: &str,
    ) {
        // Get or create Font dictionary in resources
        let font_dict = if let Ok(Object::Dictionary(dict)) = resources.get(b"Font") {
            dict.clone()
        } else {
            Dictionary::new()
        };

        let mut font_dict = font_dict;
        font_dict.set(resource_name, Object::Reference(font_id));
        resources.set("Font", font_dict);
    }

    /// Gets the number of embedded fonts
    pub fn count(&self) -> usize {
        self.fonts.len()
    }

    /// Gets an iterator over the embedded fonts
    pub fn fonts(&self) -> impl Iterator<Item = &(Font, ObjectId, String)> {
        self.fonts.iter()
    }

    /// Clears all cached fonts
    pub fn clear(&mut self) {
        self.fonts.clear();
        self.standard_fonts.clear();
        self.name_counter = 0;
    }
}

/// Helper for building text operations in PDF content streams
pub struct TextBuilder {
    operations: Vec<Operation>,
}

impl Default for TextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBuilder {
    /// Creates a new text builder
    pub fn new() -> Self {
        TextBuilder {
            operations: Vec::new(),
        }
    }

    /// Begins a text object
    pub fn begin_text(mut self) -> Self {
        self.operations.push(TextOperations::begin_text());
        self
    }

    /// Sets the font and size
    pub fn set_font(mut self, resource_name: &str, size: f32) -> Self {
        self.operations
            .push(TextOperations::set_font(resource_name, size));
        self
    }

    /// Sets the text position
    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.operations.push(TextOperations::position(x, y));
        self
    }

    /// Shows text (use `show_encoded` for composite fonts)
    pub fn show(mut self, text: &str) -> Self {
        self.operations.push(TextOperations::show(text));
        self
    }

    /// Shows text with specific encoding (for Type0 fonts)
    pub fn show_encoded(mut self, encoded_text: Vec<u8>) -> Self {
        self.operations
            .push(TextOperations::show_encoded(encoded_text));
        self
    }

    /// Moves to next line with offset
    pub fn next_line(mut self, x: f32, y: f32) -> Self {
        self.operations.push(TextOperations::next_line(x, y));
        self
    }

    /// Sets text leading (line spacing)
    pub fn set_leading(mut self, leading: f32) -> Self {
        self.operations.push(TextOperations::set_leading(leading));
        self
    }

    /// Sets character spacing
    pub fn set_char_spacing(mut self, spacing: f32) -> Self {
        self.operations
            .push(TextOperations::set_char_spacing(spacing));
        self
    }

    /// Sets word spacing
    pub fn set_word_spacing(mut self, spacing: f32) -> Self {
        self.operations
            .push(TextOperations::set_word_spacing(spacing));
        self
    }

    /// Sets horizontal scaling
    pub fn set_horizontal_scaling(mut self, scale: f32) -> Self {
        self.operations
            .push(TextOperations::set_horizontal_scaling(scale));
        self
    }

    /// Sets text rendering mode
    pub fn set_rendering_mode(mut self, mode: TextRenderingMode) -> Self {
        self.operations
            .push(TextOperations::set_rendering_mode(mode));
        self
    }

    /// Sets text rise (superscript/subscript)
    pub fn set_rise(mut self, rise: f32) -> Self {
        self.operations.push(TextOperations::set_rise(rise));
        self
    }

    /// Sets text matrix for transformations
    pub fn set_matrix(mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
        self.operations
            .push(TextOperations::set_matrix(a, b, c, d, e, f));
        self
    }

    /// Sets fill color (RGB)
    pub fn set_fill_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.operations
            .push(TextOperations::set_fill_color_rgb(r, g, b));
        self
    }

    /// Sets stroke color (RGB)
    pub fn set_stroke_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.operations
            .push(TextOperations::set_stroke_color_rgb(r, g, b));
        self
    }

    /// Ends the text object
    pub fn end_text(mut self) -> Self {
        self.operations.push(TextOperations::end_text());
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

/// Text rendering modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextRenderingMode {
    Fill = 0,
    Stroke = 1,
    FillThenStroke = 2,
    Invisible = 3,
    FillAndClip = 4,
    StrokeAndClip = 5,
    FillStrokeAndClip = 6,
    Clip = 7,
}

/// Helper struct for text operations
pub struct TextOperations;

impl TextOperations {
    /// Begin text object
    pub fn begin_text() -> Operation {
        Operation::new("BT", vec![])
    }

    /// End text object
    pub fn end_text() -> Operation {
        Operation::new("ET", vec![])
    }

    /// Set font and size
    pub fn set_font(resource_name: &str, size: f32) -> Operation {
        Operation::new(
            "Tf",
            vec![Object::Name(resource_name.as_bytes().to_vec()), size.into()],
        )
    }

    /// Set text position
    pub fn position(x: f32, y: f32) -> Operation {
        Operation::new("Td", vec![x.into(), y.into()])
    }

    /// Show text string (use show_with_font for proper encoding)
    pub fn show(text: &str) -> Operation {
        Operation::new("Tj", vec![Object::string_literal(text)])
    }

    /// Show text with proper encoding for the font type
    pub fn show_encoded(encoded_text: Vec<u8>) -> Operation {
        Operation::new(
            "Tj",
            vec![Object::String(
                encoded_text,
                lopdf::StringFormat::Hexadecimal,
            )],
        )
    }

    /// Move to next line with offset
    pub fn next_line(x: f32, y: f32) -> Operation {
        Operation::new("Td", vec![x.into(), y.into()])
    }

    /// Set text leading
    pub fn set_leading(leading: f32) -> Operation {
        Operation::new("TL", vec![leading.into()])
    }

    /// Set character spacing
    pub fn set_char_spacing(spacing: f32) -> Operation {
        Operation::new("Tc", vec![spacing.into()])
    }

    /// Set word spacing
    pub fn set_word_spacing(spacing: f32) -> Operation {
        Operation::new("Tw", vec![spacing.into()])
    }

    /// Set horizontal scaling (percentage)
    pub fn set_horizontal_scaling(scale: f32) -> Operation {
        Operation::new("Tz", vec![scale.into()])
    }

    /// Set rendering mode
    pub fn set_rendering_mode(mode: TextRenderingMode) -> Operation {
        Operation::new("Tr", vec![(mode as i32).into()])
    }

    /// Set text rise
    pub fn set_rise(rise: f32) -> Operation {
        Operation::new("Ts", vec![rise.into()])
    }

    /// Set text matrix
    pub fn set_matrix(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Operation {
        Operation::new(
            "Tm",
            vec![a.into(), b.into(), c.into(), d.into(), e.into(), f.into()],
        )
    }

    /// Set fill color RGB
    pub fn set_fill_color_rgb(r: f32, g: f32, b: f32) -> Operation {
        Operation::new("rg", vec![r.into(), g.into(), b.into()])
    }

    /// Set stroke color RGB
    pub fn set_stroke_color_rgb(r: f32, g: f32, b: f32) -> Operation {
        Operation::new("RG", vec![r.into(), g.into(), b.into()])
    }
}

/// Utility functions for common text operations
pub mod utils {
    use super::*;

    /// Represents a line of text with its content and position
    #[derive(Debug, Clone)]
    pub struct TextLine {
        pub text: String,
        pub x: f32,
        pub y: f32,
    }

    /// Word wrapping strategy
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum WrapStrategy {
        /// Break on word boundaries (default)
        Word,
        /// Break on character boundaries (for long words)
        Character,
        /// Hybrid: try word breaks, fall back to character breaks
        Hybrid,
    }

    /// Text alignment options
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TextAlign {
        Left,
        Center,
        Right,
    }

    /// Wraps text into multiple lines respecting width constraint
    pub fn wrap_text(
        font: &Font,
        text: &str,
        max_width: f32,
        font_size: f32,
        strategy: WrapStrategy,
    ) -> Vec<String> {
        if text.is_empty() {
            return vec![];
        }

        match strategy {
            WrapStrategy::Word => wrap_by_words(font, text, max_width, font_size),
            WrapStrategy::Character => wrap_by_characters(font, text, max_width, font_size),
            WrapStrategy::Hybrid => {
                let lines = wrap_by_words(font, text, max_width, font_size);
                // Check if any line is still too long
                let mut result = Vec::new();
                for line in lines {
                    let line_width = font.text_width(&line, font_size);
                    if line_width > max_width {
                        // Break this line by characters
                        result.extend(wrap_by_characters(font, &line, max_width, font_size));
                    } else {
                        result.push(line);
                    }
                }
                result
            }
        }
    }

    fn wrap_by_words(font: &Font, text: &str, max_width: f32, font_size: f32) -> Vec<String> {
        let mut lines = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        
        if words.is_empty() {
            return lines;
        }

        let mut current_line = String::new();

        for word in words {
            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_line, word)
            };

            let width = font.text_width(&test_line, font_size);

            if width > max_width && !current_line.is_empty() {
                lines.push(current_line.clone());
                current_line = word.to_string();
            } else {
                current_line = test_line;
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    fn wrap_by_characters(font: &Font, text: &str, max_width: f32, font_size: f32) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0.0;

        for ch in text.chars() {
            let char_width = font.char_width(ch, font_size);
            
            if current_width + char_width > max_width && !current_line.is_empty() {
                lines.push(current_line.clone());
                current_line.clear();
                current_width = 0.0;
            }

            current_line.push(ch);
            current_width += char_width;
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    /// Creates a multi-line text block with proper wrapping and alignment
    pub fn create_text_block(
        font_resource: &str,
        font: &Font,
        text: &str,
        x: f32,
        y: f32,
        font_size: f32,
        max_width: Option<f32>,
        max_height: Option<f32>,
        line_height: f32,
        align: TextAlign,
        wrap_strategy: WrapStrategy,
    ) -> Vec<Operation> {
        let mut operations = Vec::new();

        // Split text by explicit line breaks first
        let paragraphs: Vec<&str> = text.split('\n').collect();
        let mut all_lines = Vec::new();

        for paragraph in paragraphs {
            if let Some(width) = max_width {
                let wrapped = wrap_text(font, paragraph, width, font_size, wrap_strategy);
                all_lines.extend(wrapped);
            } else {
                all_lines.push(paragraph.to_string());
            }
        }

        // Limit lines by max_height if specified
        if let Some(max_h) = max_height {
            let max_lines = (max_h / line_height).floor() as usize;
            if all_lines.len() > max_lines {
                all_lines.truncate(max_lines);
            }
        }

        operations.push(TextOperations::begin_text());
        operations.push(TextOperations::set_font(font_resource, font_size));

        let mut current_y = y;

        for line in all_lines {
            let line_width = font.text_width(&line, font_size);
            let line_x = match align {
                TextAlign::Left => x,
                TextAlign::Center => {
                    x + (max_width.unwrap_or(line_width) - line_width) / 2.0
                }
                TextAlign::Right => {
                    x + max_width.unwrap_or(line_width) - line_width
                }
            };

            operations.push(TextOperations::position(line_x, current_y));
            
            if font.needs_utf16_encoding() {
                operations.push(TextOperations::show_encoded(font.encode_text(&line)));
            } else {
                operations.push(TextOperations::show(&line));
            }

            current_y -= line_height;
        }

        operations.push(TextOperations::end_text());
        operations
    }

    /// Creates a simple text paragraph with word wrapping
    pub fn create_paragraph(
        font_resource: &str,
        font: &Font,
        text: &str,
        x: f32,
        y: f32,
        font_size: f32,
        max_width: f32,
        line_height: f32,
    ) -> Vec<Operation> {
        let mut operations = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();

        let mut current_line = String::new();
        let mut current_y = y;

        operations.push(TextOperations::begin_text());
        operations.push(TextOperations::set_font(font_resource, font_size));

        for word in words {
            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_line, word)
            };

            let width = font.text_width(&test_line, font_size);

            if width > max_width && !current_line.is_empty() {
                // Write current line
                operations.push(TextOperations::position(x, current_y));
                if font.needs_utf16_encoding() {
                    operations.push(TextOperations::show_encoded(
                        font.encode_text(&current_line),
                    ));
                } else {
                    operations.push(TextOperations::show(&current_line));
                }
                current_y -= line_height;
                current_line = word.to_string();
            } else {
                current_line = test_line;
            }
        }

        // Write last line
        if !current_line.is_empty() {
            operations.push(TextOperations::position(x, current_y));
            if font.needs_utf16_encoding() {
                operations.push(TextOperations::show_encoded(
                    font.encode_text(&current_line),
                ));
            } else {
                operations.push(TextOperations::show(&current_line));
            }
        }

        operations.push(TextOperations::end_text());
        operations
    }

    /// Creates centered text
    pub fn create_centered_text(
        font_resource: &str,
        font: &Font,
        text: &str,
        center_x: f32,
        y: f32,
        font_size: f32,
    ) -> Vec<Operation> {
        let width = font.text_width(text, font_size);
        let x = center_x - (width / 2.0);

        vec![
            TextOperations::begin_text(),
            TextOperations::set_font(font_resource, font_size),
            TextOperations::position(x, y),
            if font.needs_utf16_encoding() {
                TextOperations::show_encoded(font.encode_text(text))
            } else {
                TextOperations::show(text)
            },
            TextOperations::end_text(),
        ]
    }

    /// Creates right-aligned text
    pub fn create_right_aligned_text(
        font_resource: &str,
        font: &Font,
        text: &str,
        right_x: f32,
        y: f32,
        font_size: f32,
    ) -> Vec<Operation> {
        let width = font.text_width(text, font_size);
        let x = right_x - width;

        vec![
            TextOperations::begin_text(),
            TextOperations::set_font(font_resource, font_size),
            TextOperations::position(x, y),
            if font.needs_utf16_encoding() {
                TextOperations::show_encoded(font.encode_text(text))
            } else {
                TextOperations::show(text)
            },
            TextOperations::end_text(),
        ]
    }
}
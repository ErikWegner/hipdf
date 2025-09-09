//! Image Integration Tests
//!
//! Tests for the image embedding functionality with various formats.

use hipdf::images::{Image, ImageBuilder, ImageManager, ImageFormat, ColorSpace, utils};
use lopdf::{content::{Content, Operation}, dictionary, Dictionary, Document, Object, Stream};

use std::fs;
use std::path::Path;

/// Directory for test assets
const TEST_ASSETS_DIR: &str = "tests/assets";
/// Directory for test outputs
const TEST_OUTPUT_DIR: &str = "tests/outputs";

fn ensure_directories() {
    if !Path::new(TEST_OUTPUT_DIR).exists() {
        fs::create_dir_all(TEST_OUTPUT_DIR).expect("Failed to create test output directory");
    }
}

fn asset_path(filename: &str) -> String {
    format!("{}/{}", TEST_ASSETS_DIR, filename)
}

#[test]
fn test_image_loading() {
    // Test loading different image formats
    let png_result = Image::from_file(asset_path("dot.png"));
    assert!(png_result.is_ok(), "Failed to load dot.png");
    
    let png = png_result.unwrap();
    assert_eq!(png.metadata.format, ImageFormat::PNG);
    assert!(png.metadata.width > 0);
    assert!(png.metadata.height > 0);

    let jpg_result = Image::from_file(asset_path("test.jpg"));
    assert!(jpg_result.is_ok(), "Failed to load test.jpg");
    
    let jpg = jpg_result.unwrap();
    assert_eq!(jpg.metadata.format, ImageFormat::JPEG);
    assert!(!jpg.metadata.has_alpha);
}

#[test]
fn test_png_transparency() {
    let png = Image::from_file(asset_path("duck.png")).expect("Failed to load duck.png");
    
    // PNG with transparency should have alpha data
    if png.metadata.has_alpha {
        assert!(png.alpha_data.is_some());
        assert!(!png.alpha_data.as_ref().unwrap().is_empty());
    }
}

#[test]
fn test_image_metadata() {
    let dot = Image::from_file(asset_path("dot.png")).expect("Failed to load dot.png");
    
    let (width, height) = dot.dimensions();
    assert!(width > 0);
    assert!(height > 0);
    
    let aspect = dot.aspect_ratio();
    assert!(aspect > 0.0);
    
    // Check color space
    assert!(
        dot.metadata.color_space == ColorSpace::DeviceRGB ||
        dot.metadata.color_space == ColorSpace::DeviceGray
    );
}

#[test]
fn test_image_manager() {
    let mut doc = Document::with_version("1.7");
    let mut manager = ImageManager::new();
    
    let image = Image::from_file(asset_path("dot.png")).expect("Failed to load image");
    
    let image_id = manager.embed_image(&mut doc, image).expect("Failed to embed image");
    
    assert_eq!(manager.count(), 1);
    
    // Test that re-embedding the same image returns the same ID
    let image2 = Image::from_file(asset_path("dot.png")).expect("Failed to load image");
    let image_id2 = manager.embed_image(&mut doc, image2).expect("Failed to embed image");
    
    assert_eq!(image_id, image_id2);
    assert_eq!(manager.count(), 1); // Should still be 1 since it's cached
}

#[test]
fn test_image_resources() {
    let mut manager = ImageManager::new();
    let mut resources = Dictionary::new();
    
    let image_id = (1, 0); // Mock object ID
    let name = manager.add_to_resources(&mut resources, image_id);
    
    assert!(name.starts_with("Im"));
    assert!(resources.has(b"XObject"));
}

#[test]
fn test_image_operations() {
    let resource_name = "Im0";
    
    // Test basic draw
    let ops = ImageManager::draw_image(resource_name, 100.0, 200.0, 50.0, 75.0);
    assert_eq!(ops.len(), 4); // q, cm, Do, Q
    
    // Test rotated draw
    let rotated_ops = ImageManager::draw_image_rotated(
        resource_name,
        100.0,
        200.0,
        50.0,
        75.0,
        45.0,
    );
    assert!(rotated_ops.len() >= 4);
}

#[test]
fn test_image_builder() {
    let builder = ImageBuilder::new()
        .add_image("Im0", 10.0, 10.0, 100.0, 100.0)
        .add_image_rotated("Im1", 120.0, 10.0, 100.0, 100.0, 30.0);
    
    let operations = builder.build();
    assert!(!operations.is_empty());
    assert!(operations.len() >= 8); // Two images, each with q, cm, Do, Q
}

/// Integration test that creates a complete PDF with multiple images
#[test]
fn test_images_integration() {
    ensure_directories();

    // Create PDF document
    let mut doc = Document::with_version("1.7");

    // Setup document structure
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Count" => 1,
    });

    // Add fonts for labels
    let helvetica = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });

    let mut resources = dictionary! {
        "Font" => dictionary! {
            "F1" => helvetica,
        },
    };

    // Create image manager
    let mut image_manager = ImageManager::new();

    // Load all test images
    let dot_png = Image::from_file(asset_path("dot.png"))
        .expect("Failed to load dot.png");
    let rect_png = Image::from_file(asset_path("duck.png"))
        .expect("Failed to load duck.png");
    let test_jpg = Image::from_file(asset_path("test.jpg"))
        .expect("Failed to load test.jpg");
    let print_jpeg = Image::from_file(asset_path("print.jpeg"))
        .expect("Failed to load print.jpeg");

    // Embed images in PDF
    let dot_id = image_manager.embed_image(&mut doc, dot_png.clone())
        .expect("Failed to embed dot.png");
    let rect_id = image_manager.embed_image(&mut doc, rect_png.clone())
        .expect("Failed to embed duck.png");
    let test_id = image_manager.embed_image(&mut doc, test_jpg.clone())
        .expect("Failed to embed test.jpg");
    let print_id = image_manager.embed_image(&mut doc, print_jpeg.clone())
        .expect("Failed to embed print.jpeg");

    // Add images to resources
    let dot_name = image_manager.add_to_resources(&mut resources, dot_id);
    let rect_name = image_manager.add_to_resources(&mut resources, rect_id);
    let test_name = image_manager.add_to_resources(&mut resources, test_id);
    let print_name = image_manager.add_to_resources(&mut resources, print_id);

    // Build page content
    let mut operations = Vec::new();

    // Title
    operations.push(Operation::new("BT", vec![]));
    operations.push(Operation::new(
        "Tf",
        vec![Object::Name(b"F1".to_vec()), 24.0.into()],
    ));
    operations.push(Operation::new("Td", vec![50.0.into(), 800.0.into()]));
    operations.push(Operation::new(
        "Tj",
        vec![Object::string_literal("Image Embedding Test")],
    ));
    operations.push(Operation::new("ET", vec![]));

    // Section 1: PNG with transparency
    operations.push(Operation::new("BT", vec![]));
    operations.push(Operation::new(
        "Tf",
        vec![Object::Name(b"F1".to_vec()), 14.0.into()],
    ));
    operations.push(Operation::new("Td", vec![50.0.into(), 750.0.into()]));
    operations.push(Operation::new(
        "Tj",
        vec![Object::string_literal("PNG Images with Transparency:")],
    ));
    operations.push(Operation::new("ET", vec![]));

    // Draw PNG images
    operations.extend(ImageManager::draw_image(&dot_name, 50.0, 600.0, 100.0, 100.0));
    operations.extend(ImageManager::draw_image(&rect_name, 170.0, 600.0, 100.0, 100.0));

    // Add labels
    operations.push(Operation::new("BT", vec![]));
    operations.push(Operation::new(
        "Tf",
        vec![Object::Name(b"F1".to_vec()), 10.0.into()],
    ));
    operations.push(Operation::new("Td", vec![50.0.into(), 590.0.into()]));
    operations.push(Operation::new(
        "Tj",
        vec![Object::string_literal("dot.png")],
    ));
    operations.push(Operation::new("ET", vec![]));

    operations.push(Operation::new("BT", vec![]));
    operations.push(Operation::new("Td", vec![170.0.into(), 590.0.into()]));
    operations.push(Operation::new(
        "Tj",
        vec![Object::string_literal("duck.png")],
    ));
    operations.push(Operation::new("ET", vec![]));

    // Section 2: JPEG images
    operations.push(Operation::new("BT", vec![]));
    operations.push(Operation::new(
        "Tf",
        vec![Object::Name(b"F1".to_vec()), 14.0.into()],
    ));
    operations.push(Operation::new("Td", vec![50.0.into(), 550.0.into()]));
    operations.push(Operation::new(
        "Tj",
        vec![Object::string_literal("JPEG Images:")],
    ));
    operations.push(Operation::new("ET", vec![]));

    // Draw JPEG images
    operations.extend(ImageManager::draw_image(&test_name, 50.0, 400.0, 100.0, 100.0));
    operations.extend(ImageManager::draw_image(&print_name, 170.0, 400.0, 100.0, 100.0));

    // Add labels
    operations.push(Operation::new("BT", vec![]));
    operations.push(Operation::new(
        "Tf",
        vec![Object::Name(b"F1".to_vec()), 10.0.into()],
    ));
    operations.push(Operation::new("Td", vec![50.0.into(), 390.0.into()]));
    operations.push(Operation::new(
        "Tj",
        vec![Object::string_literal("test.jpg")],
    ));
    operations.push(Operation::new("ET", vec![]));

    operations.push(Operation::new("BT", vec![]));
    operations.push(Operation::new("Td", vec![170.0.into(), 390.0.into()]));
    operations.push(Operation::new(
        "Tj",
        vec![Object::string_literal("print.jpeg")],
    ));
    operations.push(Operation::new("ET", vec![]));

    // Section 3: Stacked PNG Images (Transparency Test)
    operations.push(Operation::new("BT", vec![]));
    operations.push(Operation::new(
        "Tf",
        vec![Object::Name(b"F1".to_vec()), 14.0.into()],
    ));
    operations.push(Operation::new("Td", vec![50.0.into(), 350.0.into()]));
    operations.push(Operation::new(
        "Tj",
        vec![Object::string_literal("Stacked PNG Images (Transparency Test):")],
    ));
    operations.push(Operation::new("ET", vec![]));

    // Background color for transparency demonstration
    operations.push(Operation::new("0.8", vec![])); // Light gray background
    operations.push(Operation::new("0.8", vec![]));
    operations.push(Operation::new("0.8", vec![]));
    operations.push(Operation::new("rg", vec![])); // Fill RGB
    operations.extend(vec![
        Operation::new("q", vec![]),
        Operation::new("re", vec![50.0.into(), 150.0.into(), 150.0.into(), 150.0.into()]),
        Operation::new("f", vec![]), // Fill rectangle
        Operation::new("Q", vec![]),
    ]);

    // Stack PNG images on top of each other to test transparency
    // First draw duck.png as base (with red background for visibility)
    operations.push(Operation::new("1.0", vec![])); // Red background for base
    operations.push(Operation::new("0.7", vec![]));
    operations.push(Operation::new("0.7", vec![]));
    operations.push(Operation::new("rg", vec![]));
    operations.extend(vec![
        Operation::new("q", vec![]),
        Operation::new("re", vec![60.0.into(), 160.0.into(), 80.0.into(), 80.0.into()]),
        Operation::new("f", vec![]),
        Operation::new("Q", vec![]),
    ]);

    // Draw duck.png over red background
    operations.extend(ImageManager::draw_image(&rect_name, 60.0, 160.0, 80.0, 80.0));

    // Draw dot.png stacked on top (should show transparency through both)
    operations.extend(ImageManager::draw_image(&dot_name, 80.0, 180.0, 60.0, 60.0));

    // Second stack: Different order to show layering effect
    operations.push(Operation::new("0.7", vec![])); // Light blue background
    operations.push(Operation::new("0.9", vec![]));
    operations.push(Operation::new("1.0", vec![]));
    operations.push(Operation::new("rg", vec![]));
    operations.extend(vec![
        Operation::new("q", vec![]),
        Operation::new("re", vec![200.0.into(), 160.0.into(), 130.0.into(), 130.0.into()]),
        Operation::new("f", vec![]),
        Operation::new("Q", vec![]),
    ]);

    // Draw dot.png first as base
    operations.extend(ImageManager::draw_image(&dot_name, 200.0, 160.0, 130.0, 130.0));

    // Draw duck.png on top (smaller to show layering)
    operations.extend(ImageManager::draw_image(&rect_name, 220.0, 180.0, 90.0, 90.0));

    // Section 4: Transformed images
    operations.push(Operation::new("BT", vec![]));
    operations.push(Operation::new(
        "Tf",
        vec![Object::Name(b"F1".to_vec()), 14.0.into()],
    ));
    operations.push(Operation::new("Td", vec![50.0.into(), 110.0.into()]));
    operations.push(Operation::new(
        "Tj",
        vec![Object::string_literal("Transformed Images:")],
    ));
    operations.push(Operation::new("ET", vec![]));

    // Rotated images
    operations.extend(ImageManager::draw_image_rotated(
        &dot_name,
        50.0,
        50.0,
        80.0,
        80.0,
        45.0,
    ));

    operations.extend(ImageManager::draw_image_rotated(
        &rect_name,
        150.0,
        50.0,
        80.0,
        80.0,
        -30.0,
    ));

    // Scaled with aspect ratio
    operations.extend(ImageManager::draw_image_fit(
        &test_name,
        &test_jpg,
        250.0,
        50.0,
        120.0,
        80.0,
    ));

    // Section 5: Image grid
    operations.push(Operation::new("BT", vec![]));
    operations.push(Operation::new(
        "Tf",
        vec![Object::Name(b"F1".to_vec()), 14.0.into()],
    ));
    operations.push(Operation::new("Td", vec![350.0.into(), 750.0.into()]));
    operations.push(Operation::new(
        "Tj",
        vec![Object::string_literal("Thumbnail Grid:")],
    ));
    operations.push(Operation::new("ET", vec![]));

    // Create thumbnail grid
    let thumbnails = vec![
        (dot_name.clone(), &dot_png),
        (rect_name.clone(), &rect_png),
        (test_name.clone(), &test_jpg),
        (print_name.clone(), &print_jpeg),
        (dot_name.clone(), &dot_png),
        (rect_name.clone(), &rect_png),
    ];

    operations.extend(utils::create_thumbnail_grid(
        &thumbnails,
        350.0,
        700.0,
        3,
        60.0,
        10.0,
    ));

    // Print image information
    operations.push(Operation::new("BT", vec![]));
    operations.push(Operation::new(
        "Tf",
        vec![Object::Name(b"F1".to_vec()), 10.0.into()],
    ));
    operations.push(Operation::new("Td", vec![50.0.into(), 150.0.into()]));
    operations.push(Operation::new(
        "Tj",
        vec![Object::string_literal("Image Information:")],
    ));
    operations.push(Operation::new("ET", vec![]));

    // Display metadata for each image
    let images_info = vec![
        ("dot.png", &dot_png),
        ("duck.png", &rect_png),
        ("test.jpg", &test_jpg),
        ("print.jpeg", &print_jpeg),
    ];

    let mut y_pos = 130.0;
    for (name, img) in images_info {
        let (w, h) = img.dimensions();
        let info = format!(
            "{}: {}x{}, {}, {}",
            name,
            w,
            h,
            match img.metadata.format {
                ImageFormat::PNG => "PNG",
                ImageFormat::JPEG => "JPEG",
            },
            if img.metadata.has_alpha { "with alpha" } else { "no alpha" }
        );

        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new(
            "Tf",
            vec![Object::Name(b"F1".to_vec()), 9.0.into()],
        ));
        operations.push(Operation::new("Td", vec![70.0.into(), y_pos.into()]));
        operations.push(Operation::new(
            "Tj",
            vec![Object::string_literal(info)],
        ));
        operations.push(Operation::new("ET", vec![]));

        y_pos -= 15.0;
    }

    // Create content stream
    let content = Content { operations };
    let content_stream = Stream::new(dictionary! {}, content.encode().unwrap());
    let content_id = doc.add_object(content_stream);

    // Create page
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
        "Resources" => resources,
    });

    // Update pages
    doc.get_object_mut(pages_id)
        .and_then(Object::as_dict_mut)
        .unwrap()
        .set("Kids", vec![Object::Reference(page_id)]);

    // Create catalog
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    doc.trailer.set("Root", Object::Reference(catalog_id));

    // Save PDF
    let output_path = format!("{}/images_test.pdf", TEST_OUTPUT_DIR);
    doc.save(&output_path).expect("Failed to save PDF");

    assert!(Path::new(&output_path).exists());

    println!("✅ Image embedding test completed successfully");
    println!("📄 PDF created: {}", output_path);
    println!("\n🖼️ Images embedded:");
    println!("  - dot.png (PNG with transparency)");
    println!("  - duck.png (PNG with transparency)");
    println!("  - test.jpg (JPEG)");
    println!("  - print.jpeg (JPEG)");
    println!("\n📊 Features demonstrated:");
    println!("  - PNG transparency support");
    println!("  - JPEG embedding");
    println!("  - Image transformations (rotation, scaling)");
    println!("  - Aspect ratio preservation");
    println!("  - Thumbnail grid generation");
    println!("  - Image metadata display");
}

#[test]
fn test_aspect_ratio_preservation() {
    let image = Image::from_file(asset_path("test.jpg"))
        .expect("Failed to load image");
    
    let resource_name = "Im0";
    
    // Test fitting in different boxes
    let ops1 = ImageManager::draw_image_fit(
        resource_name,
        &image,
        0.0,
        0.0,
        200.0,
        100.0,
    );
    assert!(!ops1.is_empty());
    
    let ops2 = ImageManager::draw_image_fit(
        resource_name,
        &image,
        0.0,
        0.0,
        100.0,
        200.0,
    );
    assert!(!ops2.is_empty());
}

#[test]
fn test_watermark_creation() {
    let ops = utils::create_watermark("Im0", 595.0, 842.0, 0.3);
    assert!(!ops.is_empty());
    assert!(ops.len() >= 2); // At least q and Q
}

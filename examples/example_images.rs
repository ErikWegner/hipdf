use hipdf::images::{Image, ImageManager};
use hipdf::lopdf::{Document, dictionary, Object, Stream, content::Content};

fn create_image_showcase_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new PDF document
    let mut doc = Document::with_version("1.5");

    // Initialize the image manager for efficient embedding
    let mut image_manager = ImageManager::new();

    // Load various image formats to demonstrate quality preservation
    println!("Loading images...");

    // Load PNG images with different properties
    let png_rgb = Image::from_file("tests/assets/duck.png")?;
    let png_indexed = Image::from_file("tests/assets/indexed.png")?;
    let png_16bit = Image::from_file("tests/assets/16bit_test.png")?;
    let png_srgb = Image::from_file("tests/assets/srgb_profile.png")?;
    let png_transparent = Image::from_file("tests/assets/dot.png")?;

    // Load JPEG images
    let jpeg_standard = Image::from_file("tests/assets/test.jpg")?;
    let jpeg_print = Image::from_file("tests/assets/print.jpeg")?;

    println!("✅ All images loaded successfully!");
    println!("PNG RGB: {}x{} pixels", png_rgb.metadata.width, png_rgb.metadata.height);
    println!("PNG Indexed: {}x{} pixels", png_indexed.metadata.width, png_indexed.metadata.height);
    println!("PNG 16-bit: {}x{} pixels", png_16bit.metadata.width, png_16bit.metadata.height);
    println!("PNG sRGB: {}x{} pixels", png_srgb.metadata.width, png_srgb.metadata.height);
    println!("PNG Transparent: {}x{} pixels", png_transparent.metadata.width, png_transparent.metadata.height);
    println!("JPEG Standard: {}x{} pixels", jpeg_standard.metadata.width, jpeg_standard.metadata.height);
    println!("JPEG Print: {}x{} pixels", jpeg_print.metadata.width, jpeg_print.metadata.height);

    // Store images for later use in drawing (before embedding)
    let images = vec![
        png_rgb.clone(), png_indexed.clone(), png_16bit.clone(), png_srgb.clone(), png_transparent.clone(), jpeg_standard.clone(), jpeg_print.clone()
    ];

    // Embed all images in the document (with caching)
    println!("\nEmbedding images with perfect quality preservation...");

    let img1_id = image_manager.embed_image(&mut doc, png_rgb)?;
    let img2_id = image_manager.embed_image(&mut doc, png_indexed)?;
    let img3_id = image_manager.embed_image(&mut doc, png_16bit)?;
    let img4_id = image_manager.embed_image(&mut doc, png_srgb)?;
    let img5_id = image_manager.embed_image(&mut doc, png_transparent)?;
    let img6_id = image_manager.embed_image(&mut doc, jpeg_standard)?;
    let img7_id = image_manager.embed_image(&mut doc, jpeg_print)?;

    println!("✅ All images embedded successfully!");

    // Set up document structure
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Count" => 1,
    });

    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });

    // Create page resources with all embedded images
    let mut resources = dictionary! {
        "Font" => dictionary! {
            "F1" => font_id,
        },
    };

    // Add all images to resources and get their names
    let img1_name = image_manager.add_to_resources(&mut resources, img1_id);
    let img2_name = image_manager.add_to_resources(&mut resources, img2_id);
    let img3_name = image_manager.add_to_resources(&mut resources, img3_id);
    let img4_name = image_manager.add_to_resources(&mut resources, img4_id);
    let img5_name = image_manager.add_to_resources(&mut resources, img5_id);
    let img6_name = image_manager.add_to_resources(&mut resources, img6_id);
    let img7_name = image_manager.add_to_resources(&mut resources, img7_id);

    let image_names = vec![img1_name.clone(), img2_name.clone(), img3_name.clone(), img4_name.clone(), img5_name.clone(), img6_name.clone(), img7_name.clone()];

    // Build page content with various image demonstrations
    let mut content = Content { operations: Vec::new() };

    // Set font for text labels
    content.operations.push(lopdf::content::Operation::new("BT", vec![]));
    content.operations.push(lopdf::content::Operation::new("F1", vec![Object::Integer(12)]));
    content.operations.push(lopdf::content::Operation::new("Tm", vec![
        Object::Integer(1), Object::Integer(0), Object::Integer(0), Object::Integer(1),
        Object::Integer(50), Object::Integer(800)
    ]));

    // Add title
    content.operations.push(lopdf::content::Operation::new("Tj", vec![
        Object::String(b"HiPDF Image Quality Showcase - 100% Quality Preservation".to_vec(), lopdf::StringFormat::Literal)
    ]));
    content.operations.push(lopdf::content::Operation::new("ET", vec![]));

    // Section 1: PNG Images
    content.operations.push(lopdf::content::Operation::new("BT", vec![]));
    content.operations.push(lopdf::content::Operation::new("F1", vec![Object::Integer(14)]));
    content.operations.push(lopdf::content::Operation::new("Tm", vec![
        Object::Integer(1), Object::Integer(0), Object::Integer(0), Object::Integer(1),
        Object::Integer(50), Object::Integer(750)
    ]));
    content.operations.push(lopdf::content::Operation::new("Tj", vec![
        Object::String(b"PNG Images - Perfect Quality Preservation".to_vec(), lopdf::StringFormat::Literal)
    ]));
    content.operations.push(lopdf::content::Operation::new("ET", vec![]));

    // Draw PNG images with labels
    let png_images = vec![
        (0, "Standard RGB", 50.0, 650.0, 150.0, 100.0),
        (1, "Indexed Colors", 250.0, 650.0, 150.0, 100.0),
        (2, "16-bit Depth", 450.0, 650.0, 150.0, 100.0),
    ];

    for (img_idx, label, x, y, w, h) in png_images {
        let img_name = &image_names[img_idx];
        let img = &images[img_idx];

        // Add label
        content.operations.push(lopdf::content::Operation::new("BT", vec![]));
        content.operations.push(lopdf::content::Operation::new("F1", vec![Object::Integer(10)]));
        content.operations.push(lopdf::content::Operation::new("Tm", vec![
            Object::Integer(1), Object::Integer(0), Object::Integer(0), Object::Integer(1),
            Object::Real(x), Object::Real(y - 15.0)
        ]));
        content.operations.push(lopdf::content::Operation::new("Tj", vec![
            Object::String(label.as_bytes().to_vec(), lopdf::StringFormat::Literal)
        ]));
        content.operations.push(lopdf::content::Operation::new("ET", vec![]));

        // Draw image
        content.operations.extend(ImageManager::draw_image_fit(img_name, img, x, y, w, h));
    }

    // Section 2: Special PNG Features
    content.operations.push(lopdf::content::Operation::new("BT", vec![]));
    content.operations.push(lopdf::content::Operation::new("F1", vec![Object::Integer(14)]));
    content.operations.push(lopdf::content::Operation::new("Tm", vec![
        Object::Integer(1), Object::Integer(0), Object::Integer(0), Object::Integer(1),
        Object::Integer(50), Object::Integer(500)
    ]));
    content.operations.push(lopdf::content::Operation::new("Tj", vec![
        Object::String(b"Special PNG Features - ICC Profiles & Transparency".to_vec(), lopdf::StringFormat::Literal)
    ]));
    content.operations.push(lopdf::content::Operation::new("ET", vec![]));

    // Draw special PNG images
    let special_images = vec![
        (3, "sRGB Profile", 50.0, 400.0, 150.0, 100.0),
        (4, "Transparent", 250.0, 400.0, 150.0, 100.0),
    ];

    for (img_idx, label, x, y, w, h) in special_images {
        let img_name = &image_names[img_idx];
        let img = &images[img_idx];

        // Add label
        content.operations.push(lopdf::content::Operation::new("BT", vec![]));
        content.operations.push(lopdf::content::Operation::new("F1", vec![Object::Integer(10)]));
        content.operations.push(lopdf::content::Operation::new("Tm", vec![
            Object::Integer(1), Object::Integer(0), Object::Integer(0), Object::Integer(1),
            Object::Real(x), Object::Real(y - 15.0)
        ]));
        content.operations.push(lopdf::content::Operation::new("Tj", vec![
            Object::String(label.as_bytes().to_vec(), lopdf::StringFormat::Literal)
        ]));
        content.operations.push(lopdf::content::Operation::new("ET", vec![]));

        // Draw image
        content.operations.extend(ImageManager::draw_image_fit(img_name, img, x, y, w, h));
    }

    // Section 3: JPEG Images
    content.operations.push(lopdf::content::Operation::new("BT", vec![]));
    content.operations.push(lopdf::content::Operation::new("F1", vec![Object::Integer(14)]));
    content.operations.push(lopdf::content::Operation::new("Tm", vec![
        Object::Integer(1), Object::Integer(0), Object::Integer(0), Object::Integer(1),
        Object::Integer(50), Object::Integer(250)
    ]));
    content.operations.push(lopdf::content::Operation::new("Tj", vec![
        Object::String(b"JPEG Images - Optimized Compression".to_vec(), lopdf::StringFormat::Literal)
    ]));
    content.operations.push(lopdf::content::Operation::new("ET", vec![]));

    // Draw JPEG images
    let jpeg_images = vec![
        (5, "Standard JPEG", 50.0, 150.0, 150.0, 100.0),
        (6, "Print Quality", 250.0, 150.0, 150.0, 100.0),
    ];

    for (img_idx, label, x, y, w, h) in jpeg_images {
        let img_name = &image_names[img_idx];
        let img = &images[img_idx];

        // Add label
        content.operations.push(lopdf::content::Operation::new("BT", vec![]));
        content.operations.push(lopdf::content::Operation::new("F1", vec![Object::Integer(10)]));
        content.operations.push(lopdf::content::Operation::new("Tm", vec![
            Object::Integer(1), Object::Integer(0), Object::Integer(0), Object::Integer(1),
            Object::Real(x), Object::Real(y - 15.0)
        ]));
        content.operations.push(lopdf::content::Operation::new("Tj", vec![
            Object::String(label.as_bytes().to_vec(), lopdf::StringFormat::Literal)
        ]));
        content.operations.push(lopdf::content::Operation::new("ET", vec![]));

        // Draw image
        content.operations.extend(ImageManager::draw_image_fit(img_name, img, x, y, w, h));
    }

    // Add quality preservation note
    content.operations.push(lopdf::content::Operation::new("BT", vec![]));
    content.operations.push(lopdf::content::Operation::new("F1", vec![Object::Integer(10)]));
    content.operations.push(lopdf::content::Operation::new("Tm", vec![
        Object::Integer(1), Object::Integer(0), Object::Integer(0), Object::Integer(1),
        Object::Integer(50), Object::Integer(50)
    ]));
    content.operations.push(lopdf::content::Operation::new("Tj", vec![
        Object::String(b"Note: All images embedded with 100% quality preservation - no lossy compression applied!".to_vec(), lopdf::StringFormat::Literal)
    ]));
    content.operations.push(lopdf::content::Operation::new("ET", vec![]));

    // Create the page
    let content_stream = Stream::new(dictionary! {}, content.encode()?);
    let content_id = doc.add_object(content_stream);

    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
        "Resources" => resources,
    });

    // Finalize document structure
    let pages_dict = doc.get_object_mut(pages_id)
        .and_then(Object::as_dict_mut)
        .unwrap();
    pages_dict.set("Kids", vec![Object::Reference(page_id)]);

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    doc.trailer.set("Root", Object::Reference(catalog_id));

    // Save the resulting PDF
    doc.save("image_showcase_example.pdf")?;

    println!("\n✅ Image showcase PDF created successfully!");
    println!("📄 Output: image_showcase_example.pdf");
    println!("📊 Images embedded: 7 (PNG: 5, JPEG: 2)");
    println!("🎨 Features demonstrated:");
    println!("   • Perfect quality preservation");
    println!("   • Multiple PNG formats (RGB, Indexed, 16-bit, sRGB, Transparent)");
    println!("   • JPEG optimization");
    println!("   • Efficient image caching");
    println!("   • Automatic resource management");

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    create_image_showcase_example()
}

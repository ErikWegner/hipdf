//! Basic usage example for hipdf library
//!
//! This example demonstrates the core functionality of the library
//! including PDF creation, OCG layers, and image embedding.

use hipdf::ocg::{OCGManager, Layer};
use hipdf::lopdf::{Document, content::{Content, Operation}, dictionary};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Creating basic PDF with hipdf...");

    // Create a new PDF document
    let mut doc = Document::with_version("1.7");

    // Create OCG (Optional Content Groups) manager
    let mut ocg_manager = OCGManager::with_config(Default::default());

    // Add some layers
    ocg_manager.add_layer(Layer::new("Background", true));
    ocg_manager.add_layer(Layer::new("Main Content", true));
    ocg_manager.add_layer(Layer::new("Overlay", false));

    // Initialize layers in the document
    ocg_manager.initialize(&mut doc);

    // Create a page with content
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
    });

    // Create page content
    let mut operations = Vec::new();

    // Add some text content
    operations.extend(vec![
        Operation::new("BT", vec![]),
        Operation::new("Tf", vec![
            hipdf::lopdf::Object::Name(b"F1".to_vec()),
            12.into()
        ]),
        Operation::new("Td", vec![50.into(), 750.into()]),
        Operation::new("Tj", vec![hipdf::lopdf::Object::string_literal("Hello from hipdf!")]),
        Operation::new("ET", vec![]),
    ]);

    // Add a rectangle
    operations.extend(vec![
        Operation::new("q", vec![]),
        Operation::new("0.8", vec![]),
        Operation::new("w", vec![]),
        Operation::new("re", vec![100.into(), 700.into(), 200.into(), 100.into()]),
        Operation::new("S", vec![]),
        Operation::new("Q", vec![]),
    ]);

    let content = Content { operations };
    let content_id = doc.add_object(hipdf::lopdf::Stream::new(dictionary! {}, content.encode()?));

    // Create page
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
    });

    // Update pages
    doc.get_object_mut(pages_id)
        .and_then(hipdf::lopdf::Object::as_dict_mut)
        .unwrap()
        .set("Kids", vec![hipdf::lopdf::Object::Reference(page_id)]);

    doc.get_object_mut(pages_id)
        .and_then(hipdf::lopdf::Object::as_dict_mut)
        .unwrap()
        .set("Count", 1);

    // Create catalog
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => hipdf::lopdf::Object::Reference(pages_id),
    });

    doc.trailer.set("Root", hipdf::lopdf::Object::Reference(catalog_id));

    // Save the document
    let output_path = "examples/basic_example.pdf";
    doc.save(output_path)?;

    println!("✅ Basic PDF created successfully: {}", output_path);
    println!("📊 Layers created: {}", ocg_manager.len());
    println!("🔧 Features tested:");
    println!("   • PDF document creation");
    println!("   • Optional Content Groups (layers)");
    println!("   • Content operations (text, shapes)");
    println!("   • Document structure");

    Ok(())
}
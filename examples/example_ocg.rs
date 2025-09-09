use hipdf::ocg::{Layer, LayerContentBuilder, LayerOperations as Ops, OCGConfig, OCGManager};
use hipdf::lopdf::{Document, dictionary, Object, Stream, content::Content};

fn create_layered_pdf_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new PDF document
    let mut doc = Document::with_version("1.5");
    
    // 1. Create and configure the OCG manager
    let config = OCGConfig {
        base_state: "ON".to_string(),
        create_panel_ui: true,
        intent: vec!["View".to_string(), "Design".to_string()],
    };
    let mut ocg_manager = OCGManager::with_config(config);
    
    // 2. Define your layers
    ocg_manager.add_layer(Layer::new("Background", true));     // visible by default
    ocg_manager.add_layer(Layer::new("Main Content", true));   // visible by default  
    ocg_manager.add_layer(Layer::new("Annotations", false));   // hidden by default
    ocg_manager.add_layer(Layer::new("Watermark", false));     // hidden by default
    
    // 3. Initialize layers in the document
    ocg_manager.initialize(&mut doc);
    
    // 4. Setup document structure and fonts
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Count" => 1,
    });
    
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });
    
    // 5. Setup page resources
    let mut resources = dictionary! {
        "Font" => dictionary! {
            "F1" => font_id,
        },
    };
    
    let layer_tags = ocg_manager.setup_page_resources(&mut resources);
    
    // 6. Build the page content using LayerContentBuilder
    let mut builder = LayerContentBuilder::new();
    
    // Background layer - light blue background
    if let Some(bg_tag) = layer_tags.get(&"Background".to_string()) {
        builder
            .begin_layer(bg_tag)
            .add_operation(Ops::set_fill_color_rgb(0.9, 0.95, 1.0))
            .add_operation(Ops::rectangle(0.0, 0.0, 595.0, 842.0))
            .add_operation(Ops::fill())
            .end_layer();
    }
    
    // Main content layer - title and text
    if let Some(content_tag) = layer_tags.get(&"Main Content".to_string()) {
        builder
            .begin_layer(content_tag)
            .add_operation(Ops::begin_text())
            .add_operation(Ops::set_fill_color_gray(0.0))
            .add_operation(Ops::set_font("F1", 24.0))
            .add_operation(Ops::text_position(50.0, 750.0))
            .add_operation(Ops::show_text("Layered PDF Document"))
            .add_operation(Ops::end_text())
            // Add a green rectangle
            .add_operation(Ops::set_fill_color_rgb(0.2, 0.6, 0.2))
            .add_operation(Ops::rectangle(50.0, 550.0, 200.0, 100.0))
            .add_operation(Ops::fill())
            .end_layer();
    }
    
    // Annotations layer - red annotations (hidden by default)
    if let Some(anno_tag) = layer_tags.get(&"Annotations".to_string()) {
        builder
            .begin_layer(anno_tag)
            .add_operation(Ops::set_stroke_color_rgb(1.0, 0.0, 0.0))
            .add_operation(Ops::rectangle(260.0, 560.0, 80.0, 80.0))
            .add_operation(Ops::stroke())
            .add_operation(Ops::begin_text())
            .add_operation(Ops::set_fill_color_rgb(1.0, 0.0, 0.0))
            .add_operation(Ops::set_font("F1", 12.0))
            .add_operation(Ops::text_position(350.0, 590.0))
            .add_operation(Ops::show_text("Important!"))
            .add_operation(Ops::end_text())
            .end_layer();
    }
    
    // Watermark layer - large draft text (hidden by default)
    if let Some(watermark_tag) = layer_tags.get(&"Watermark".to_string()) {
        builder
            .begin_layer(watermark_tag)
            .add_operation(Ops::begin_text())
            .add_operation(Ops::set_fill_color_gray(0.8))
            .add_operation(Ops::set_font("F1", 48.0))
            .add_operation(Ops::text_position(150.0, 400.0))
            .add_operation(Ops::show_text("DRAFT"))
            .add_operation(Ops::end_text())
            .end_layer();
    }
    
    // Content that's always visible (not in any layer)
    builder
        .add_operation(Ops::begin_text())
        .add_operation(Ops::set_fill_color_gray(0.0))
        .add_operation(Ops::set_font("F1", 10.0))
        .add_operation(Ops::text_position(50.0, 100.0))
        .add_operation(Ops::show_text("This text is always visible"))
        .add_operation(Ops::end_text());
    
    // 7. Create the content stream
    let operations = builder.build();
    let content = Content { operations };
    let content_stream = Stream::new(dictionary! {}, content.encode()?);
    let content_id = doc.add_object(content_stream);
    
    // 8. Create the page
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
        "Resources" => resources,
    });
    
    // 9. Update pages dictionary
    let pages_dict = doc.get_object_mut(pages_id)
        .and_then(Object::as_dict_mut)
        .unwrap();
    pages_dict.set("Kids", vec![Object::Reference(page_id)]);
    
    // 10. Create catalog
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    doc.trailer.set("Root", Object::Reference(catalog_id));
    
    // 11. Update the catalog with OCG properties
    ocg_manager.update_catalog(&mut doc);
    
    // 12. Save the PDF
    doc.save("layered_document.pdf")?;
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    create_layered_pdf_example()
}
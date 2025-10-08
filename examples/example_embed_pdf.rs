use hipdf::embed_pdf::{EmbedOptions, GridFillOrder, MultiPageLayout, PageRange, PdfEmbedder};
use hipdf::lopdf::{content::Content, dictionary, Document, Object, Stream};
use std::collections::HashMap;

fn embed_pdf_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new PDF document
    let mut doc = Document::with_version("1.5");

    // Initialize the PDF embedder
    let mut embedder = PdfEmbedder::new();

    // Load the source PDF you want to embed
    let source_pdf = embedder.load_pdf("source_document.pdf")?;

    // Example 1: Simple single page embedding
    let single_page_options = EmbedOptions::new()
        .at_position(50.0, 700.0) // x, y position
        .with_max_size(200.0, 200.0) // max width, height
        .with_page_range(PageRange::Single(0)); // embed first page only

    // Example 2: Grid layout with multiple pages
    let grid_options = EmbedOptions::new()
        .at_position(50.0, 400.0)
        .with_max_size(100.0, 120.0) // size per page
        .with_layout(MultiPageLayout::Grid {
            columns: 3,
            gap_x: 10.0,
            gap_y: 15.0,
            fill_order: GridFillOrder::RowFirst,
        })
        .with_page_range(PageRange::Range(0, 5)); // first 6 pages

    // Example 3: Horizontal layout with scaling
    let horizontal_options = EmbedOptions::new()
        .at_position(50.0, 200.0)
        .with_scale(0.2) // 20% of original size
        .with_layout(MultiPageLayout::Horizontal { gap: 20.0 })
        .with_page_range(PageRange::Pages(vec![0, 2, 4])); // specific pages

    // Example 4: Custom rotation and positioning
    let rotated_options = EmbedOptions::new()
        .at_position(300.0, 500.0)
        .with_scale(0.15)
        .with_rotation(45.0) // 45 degree rotation
        .with_layout(MultiPageLayout::FirstPageOnly);

    // Set up document structure
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Count" => 1,
    });

    let mut page_ops = Vec::new();
    let mut all_xobjects = HashMap::new();

    // Embed the PDFs and collect operations
    let result = embedder.embed_pdf(&mut doc, &source_pdf, &single_page_options)?;
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    let result = embedder.embed_pdf(&mut doc, &source_pdf, &grid_options)?;
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    let result = embedder.embed_pdf(&mut doc, &source_pdf, &horizontal_options)?;
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    let result = embedder.embed_pdf(&mut doc, &source_pdf, &rotated_options)?;
    page_ops.extend(result.operations);
    all_xobjects.extend(result.xobject_resources);

    // Create page content and resources
    let content = Content {
        operations: page_ops,
    };
    let content_stream = Stream::new(dictionary! {}, content.encode()?);
    let content_id = doc.add_object(content_stream);

    let mut xobject_dict = lopdf::Dictionary::new();
    for (name, obj_ref) in all_xobjects {
        xobject_dict.set(name, obj_ref);
    }

    let page_resources = dictionary! {
        "XObject" => xobject_dict,
    };

    // Create the page
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
        "Resources" => page_resources,
    });

    // Finalize document structure
    let pages_dict = doc
        .get_object_mut(pages_id)
        .and_then(Object::as_dict_mut)
        .unwrap();
    pages_dict.set("Kids", vec![Object::Reference(page_id)]);

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    doc.trailer.set("Root", Object::Reference(catalog_id));

    // Save the resulting PDF
    doc.save("output_with_embedded_pdfs.pdf")?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    embed_pdf_example()
}

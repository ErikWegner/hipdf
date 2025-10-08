//! This example showcases the font handling capabilities of the library.
//!
//! It demonstrates:
//! 1. Loading and using standard PDF fonts (like Helvetica).
//! 2. Loading and embedding custom .ttf/.otf fonts from files.
//! 3. Using the `FontManager` to manage fonts in the document.
//! 4. Using the `TextBuilder` to create complex text layouts with different
//!    styles, colors, and rendering modes.
//! 5. Using the high-level `utils` functions for common tasks like creating
//!    paragraphs with word wrapping and aligning text.
//!
//! **To run this example:**
//! 1. Create a directory `examples/assets/fonts/`.
//! 2. Place the following font files inside it:
//!    - `Inter-Variable.ttf`
//!    - `JetBrainsMono-Variable.ttf`
//! These can be downloaded from Google Fonts.

use hipdf::fonts::{utils, Font, FontManager, StandardFont, TextBuilder, TextRenderingMode};
use hipdf::lopdf::{content::Content, dictionary, Document, Object, Stream};

fn create_font_showcase_pdf() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a new PDF document and a FontManager
    let mut doc = Document::with_version("1.7");
    let mut font_manager = FontManager::new();

    // 2. Load the fonts you want to use
    // Load a standard PDF font
    let helvetica_bold = Font::standard(StandardFont::HelveticaBold);

    // Load custom fonts from files. The `Font` struct must be cloned before
    // embedding if you want to use it later for text measurement.
    let inter_font = Font::from_file("examples/assets/fonts/Inter-Variable.ttf")
        .expect("Failed to load Inter font. Make sure it's in examples/assets/fonts/");
    let jetbrains_font = Font::from_file("examples/assets/fonts/JetBrainsMono-Variable.ttf")
        .expect("Failed to load JetBrains Mono. Make sure it's in examples/assets/fonts/");

    // 3. Embed fonts into the document using the manager
    // The manager returns a resource name (e.g., "F0", "F1") to refer to the font.
    let (_, helvetica_res_name) = font_manager.embed_font(&mut doc, helvetica_bold)?;
    let (_, inter_res_name) = font_manager.embed_font(&mut doc, inter_font.clone())?;
    let (_, jetbrains_res_name) = font_manager.embed_font(&mut doc, jetbrains_font.clone())?;

    // 4. Setup document structure (Pages and Page)
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Count" => 1,
    });

    // 5. Create a resource dictionary and add all embedded fonts to it
    let mut resources = dictionary! {};
    for (_font, font_id, resource_name) in font_manager.fonts() {
        font_manager.add_to_resources(&mut resources, *font_id, resource_name);
    }

    // 6. Build the page content
    let mut operations = Vec::new();

    // --- Section 1: Title using TextBuilder ---
    operations.extend(
        TextBuilder::new()
            .begin_text()
            .set_font(&helvetica_res_name, 24.0)
            .position(50.0, 780.0)
            .show("Font Showcase")
            .end_text()
            .build(),
    );

    // --- Section 2: Simple text with different fonts ---
    let simple_text = "The quick brown fox jumps over the lazy dog.";
    operations.extend(
        TextBuilder::new()
            .begin_text()
            .set_font(&inter_res_name, 14.0)
            .position(50.0, 720.0)
            .show_encoded(inter_font.encode_text(&format!("Inter: {}", simple_text)))
            .next_line(0.0, -20.0) // Move down 20 points for the next line
            .set_font(&jetbrains_res_name, 12.0)
            .show_encoded(jetbrains_font.encode_text(&format!("JetBrains Mono: {}", simple_text)))
            .end_text()
            .build(),
    );

    // --- Section 3: Paragraph with word wrapping using a utility function ---
    let paragraph_text = "This is a longer paragraph that demonstrates the automatic word-wrapping feature provided by the `create_paragraph` utility function. It makes handling blocks of text much simpler by calculating line breaks based on a maximum width.";
    operations.extend(utils::create_paragraph(
        &inter_res_name,
        &inter_font,
        paragraph_text,
        50.0,      // x position
        650.0,     // y position
        12.0,      // font size
        500.0,     // max width
        15.0,      // line height
    ));

    // --- Section 4: Text alignment utilities ---
    let aligned_text = "Aligned Text Example";
    operations.extend(utils::create_centered_text(
        &inter_res_name,
        &inter_font,
        aligned_text,
        297.5, // Center of an A4 page (595 / 2)
        550.0,
        16.0,
    ));
    operations.extend(utils::create_right_aligned_text(
        &inter_res_name,
        &inter_font,
        aligned_text,
        545.0, // Right margin of an A4 page (595 - 50)
        520.0,
        16.0,
    ));

    // --- Section 5: Advanced styling with TextBuilder ---
    operations.extend(
        TextBuilder::new()
            .begin_text()
            // Red text
            .set_fill_color(0.8, 0.1, 0.1)
            .set_font(&inter_res_name, 14.0)
            .position(50.0, 480.0)
            .show_encoded(inter_font.encode_text("This text is red."))
            // Blue, stroked text
            .set_rendering_mode(TextRenderingMode::Stroke)
            .set_stroke_color(0.1, 0.1, 0.8)
            .next_line(0.0, -25.0)
            .show_encoded(inter_font.encode_text("This text is stroked and blue."))
            // Fill and stroke
            .set_rendering_mode(TextRenderingMode::FillThenStroke)
            .set_fill_color(1.0, 0.9, 0.2) // Yellow fill
            .set_stroke_color(0.8, 0.1, 0.1) // Red stroke
            .next_line(0.0, -25.0)
            .show_encoded(inter_font.encode_text("Fill, then stroke!"))
            .end_text()
            .build(),
    );

    // --- Section 6: Unicode and special characters ---
    let unicode_text = "Unicode: αβγδε, áéíóú, こんにちは, 🚀";
    let code_text = "fn main() { println!(\"Hello, PDF!\"); }";
    operations.extend(
        TextBuilder::new()
            .begin_text()
            .set_rendering_mode(TextRenderingMode::Fill) // Reset rendering mode
            .set_fill_color(0.0, 0.0, 0.0) // Reset to black
            .set_font(&inter_res_name, 14.0)
            .position(50.0, 380.0)
            .show_encoded(inter_font.encode_text(unicode_text))
            .next_line(0.0, -25.0)
            .set_font(&jetbrains_res_name, 12.0)
            .show_encoded(jetbrains_font.encode_text(code_text))
            .end_text()
            .build(),
    );

    // 7. Create the content stream from the collected operations
    let content = Content { operations };
    let content_stream = Stream::new(dictionary! {}, content.encode()?);
    let content_id = doc.add_object(content_stream);

    // 8. Create the page object
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
        "Resources" => resources,
    });

    // 9. Update the pages dictionary with the new page
    let pages_dict = doc
        .get_object_mut(pages_id)
        .and_then(Object::as_dict_mut)
        .unwrap();
    pages_dict.set("Kids", vec![Object::Reference(page_id)]);

    // 10. Create the document catalog
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });
    doc.trailer.set("Root", Object::Reference(catalog_id));

    // 11. Save the PDF to a file
    doc.save("fonts_showcase.pdf")?;

    println!("✅ PDF saved to fonts_showcase.pdf");

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    create_font_showcase_pdf()
}
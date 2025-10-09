//! Font Integration Tests
//!
//! These tests validate the font functionality by creating actual PDF files
//! with various fonts and text operations.

use hipdf::fonts::{
    Font, FontManager, StandardFont, TextBuilder, TextOperations as Ops, TextRenderingMode,
};
use hipdf::lopdf::content::Operation;
use hipdf::lopdf::{content::Content, dictionary, Document, Object, Stream};

use std::fs;
use std::path::Path;

const TEST_OUTPUT_DIR: &str = "tests/outputs";

fn ensure_output_dir() {
    if !Path::new(TEST_OUTPUT_DIR).exists() {
        fs::create_dir_all(TEST_OUTPUT_DIR).expect("Failed to create test output directory");
    }
}

#[test]
fn test_standard_fonts() {
    let fonts = vec![
        StandardFont::Helvetica,
        StandardFont::HelveticaBold,
        StandardFont::TimesRoman,
        StandardFont::Courier,
    ];

    for font in fonts {
        let f = Font::standard(font);
        assert_eq!(f.family(), font.family());
        assert_eq!(f.is_bold(), font.is_bold());
        assert_eq!(f.is_italic(), font.is_italic());
    }
}

#[test]
fn test_font_manager_creation() {
    let manager = FontManager::new();
    assert_eq!(manager.count(), 0);
}

#[test]
fn test_text_builder() {
    let builder = TextBuilder::new()
        .begin_text()
        .set_font("F1", 12.0)
        .position(100.0, 700.0)
        .show("Hello, World!")
        .end_text();

    let operations = builder.build();
    assert_eq!(operations.len(), 5);
}

#[test]
fn test_text_operations() {
    let ops = vec![
        Ops::begin_text(),
        Ops::set_font("F1", 12.0),
        Ops::position(10.0, 20.0),
        Ops::show("Test"),
        Ops::end_text(),
        Ops::set_leading(14.0),
        Ops::set_char_spacing(0.5),
        Ops::set_word_spacing(1.0),
    ];

    assert_eq!(ops.len(), 8);
}

#[test]
fn test_text_rendering_modes() {
    let modes = vec![
        TextRenderingMode::Fill,
        TextRenderingMode::Stroke,
        TextRenderingMode::FillThenStroke,
        TextRenderingMode::Invisible,
    ];

    for mode in modes {
        let op = Ops::set_rendering_mode(mode);
        assert!(format!("{:?}", op).contains("Tr"));
    }
}

#[test]
fn test_standard_fonts_pdf() {
    ensure_output_dir();

    let mut doc = Document::with_version("1.7");
    let mut font_manager = FontManager::new();

    // Setup document structure
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Count" => 1,
    });

    // Create fonts
    let helvetica = Font::standard(StandardFont::Helvetica);
    let helvetica_bold = Font::standard(StandardFont::HelveticaBold);
    let times = Font::standard(StandardFont::TimesRoman);
    let courier = Font::standard(StandardFont::Courier);

    let (_, f1) = font_manager.embed_font(&mut doc, helvetica).unwrap();
    let (_, f2) = font_manager.embed_font(&mut doc, helvetica_bold).unwrap();
    let (_, f3) = font_manager.embed_font(&mut doc, times).unwrap();
    let (_, f4) = font_manager.embed_font(&mut doc, courier).unwrap();

    // Build content
    let mut operations = Vec::new();

    // Title
    operations.extend(
        TextBuilder::new()
            .begin_text()
            .set_font(&f2, 24.0)
            .set_fill_color(0.0, 0.0, 0.0)
            .position(50.0, 750.0)
            .show("Standard PDF Fonts Test")
            .end_text()
            .build(),
    );

    // Helvetica
    operations.extend(
        TextBuilder::new()
            .begin_text()
            .set_font(&f1, 14.0)
            .position(50.0, 700.0)
            .show("Helvetica: The quick brown fox jumps over the lazy dog")
            .end_text()
            .build(),
    );

    // Helvetica Bold
    operations.extend(
        TextBuilder::new()
            .begin_text()
            .set_font(&f2, 14.0)
            .position(50.0, 675.0)
            .show("Helvetica Bold: The quick brown fox jumps over the lazy dog")
            .end_text()
            .build(),
    );

    // Times Roman
    operations.extend(
        TextBuilder::new()
            .begin_text()
            .set_font(&f3, 14.0)
            .position(50.0, 650.0)
            .show("Times Roman: The quick brown fox jumps over the lazy dog")
            .end_text()
            .build(),
    );

    // Courier
    operations.extend(
        TextBuilder::new()
            .begin_text()
            .set_font(&f4, 14.0)
            .position(50.0, 625.0)
            .show("Courier: The quick brown fox jumps over the lazy dog")
            .end_text()
            .build(),
    );

    // Different sizes
    operations.extend(
        TextBuilder::new()
            .begin_text()
            .set_font(&f1, 8.0)
            .position(50.0, 580.0)
            .show("8pt: Small text")
            .end_text()
            .build(),
    );

    operations.extend(
        TextBuilder::new()
            .begin_text()
            .set_font(&f1, 12.0)
            .position(50.0, 560.0)
            .show("12pt: Normal text")
            .end_text()
            .build(),
    );

    operations.extend(
        TextBuilder::new()
            .begin_text()
            .set_font(&f1, 18.0)
            .position(50.0, 535.0)
            .show("18pt: Large text")
            .end_text()
            .build(),
    );

    // Colors
    operations.extend(
        TextBuilder::new()
            .begin_text()
            .set_font(&f2, 16.0)
            .set_fill_color(1.0, 0.0, 0.0)
            .position(50.0, 490.0)
            .show("Red text")
            .end_text()
            .build(),
    );

    operations.extend(
        TextBuilder::new()
            .begin_text()
            .set_font(&f2, 16.0)
            .set_fill_color(0.0, 0.0, 1.0)
            .position(50.0, 465.0)
            .show("Blue text")
            .end_text()
            .build(),
    );

    // Rendering modes
    operations.extend(
        TextBuilder::new()
            .begin_text()
            .set_font(&f2, 20.0)
            .set_rendering_mode(TextRenderingMode::Stroke)
            .set_stroke_color(0.0, 0.5, 0.0)
            .position(50.0, 420.0)
            .show("Stroked text")
            .end_text()
            .build(),
    );

    operations.extend(
        TextBuilder::new()
            .begin_text()
            .set_font(&f2, 20.0)
            .set_rendering_mode(TextRenderingMode::FillThenStroke)
            .set_fill_color(1.0, 1.0, 0.0)
            .set_stroke_color(1.0, 0.0, 0.0)
            .position(50.0, 390.0)
            .show("Fill and Stroke")
            .end_text()
            .build(),
    );

    // Character and word spacing
    operations.extend(
        TextBuilder::new()
            .begin_text()
            .set_font(&f1, 12.0)
            .set_char_spacing(2.0)
            .position(50.0, 340.0)
            .show("Character spacing")
            .end_text()
            .build(),
    );

    operations.extend(
        TextBuilder::new()
            .begin_text()
            .set_font(&f1, 12.0)
            .set_char_spacing(0.0)
            .set_word_spacing(10.0)
            .position(50.0, 315.0)
            .show("Word spacing example")
            .end_text()
            .build(),
    );

    // Horizontal scaling
    operations.extend(
        TextBuilder::new()
            .begin_text()
            .set_font(&f1, 12.0)
            .set_horizontal_scaling(150.0)
            .position(50.0, 280.0)
            .show("Horizontally scaled to 150%")
            .end_text()
            .build(),
    );

    // Create content stream
    let content = Content { operations };
    let content_stream = Stream::new(dictionary! {}, content.encode().unwrap());
    let content_id = doc.add_object(content_stream);

    // Create resources with fonts
    let mut resources = dictionary! {};
    for (_font, font_id, resource_name) in font_manager.fonts() {
        font_manager.add_to_resources(&mut resources, *font_id, resource_name);
    }

    // Create page
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
        "Resources" => resources,
    });

    // Update pages
    if let Ok(Object::Dictionary(ref mut pages)) = doc.get_object_mut(pages_id) {
        pages.set("Kids", vec![Object::Reference(page_id)]);
    }

    // Create catalog
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });

    doc.trailer.set("Root", Object::Reference(catalog_id));

    // Save
    let output_path = format!("{}/fonts_standard_test.pdf", TEST_OUTPUT_DIR);
    doc.save(&output_path).expect("Failed to save PDF");

    assert!(Path::new(&output_path).exists());
    println!("✅ Standard fonts PDF created: {}", output_path);
}

#[test]
fn test_text_builder_comprehensive() {
    ensure_output_dir();

    let mut doc = Document::with_version("1.7");
    let mut font_manager = FontManager::new();

    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Count" => 1,
    });

    let font = Font::standard(StandardFont::Helvetica);
    let (_, font_name) = font_manager.embed_font(&mut doc, font).unwrap();

    // Use TextBuilder for everything
    let operations = TextBuilder::new()
        .begin_text()
        .set_font(&font_name, 16.0)
        .set_fill_color(0.0, 0.0, 0.0)
        .position(50.0, 750.0)
        .show("TextBuilder Example")
        .next_line(0.0, -30.0)
        .show("Second line with next_line()")
        .set_leading(20.0)
        .next_line(0.0, -30.0)
        .show("Third line with leading")
        .set_rise(5.0)
        .show(" superscript")
        .set_rise(0.0)
        .end_text()
        .build();

    let content = Content { operations };
    let content_stream = Stream::new(dictionary! {}, content.encode().unwrap());
    let content_id = doc.add_object(content_stream);

    let mut resources = dictionary! {};
    for (_font, font_id, resource_name) in font_manager.fonts() {
        font_manager.add_to_resources(&mut resources, *font_id, resource_name);
    }

    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
        "Resources" => resources,
    });

    if let Ok(Object::Dictionary(ref mut pages)) = doc.get_object_mut(pages_id) {
        pages.set("Kids", vec![Object::Reference(page_id)]);
    }

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });

    doc.trailer.set("Root", Object::Reference(catalog_id));

    let output_path = format!("{}/fonts_builder_test.pdf", TEST_OUTPUT_DIR);
    doc.save(&output_path).expect("Failed to save PDF");

    assert!(Path::new(&output_path).exists());
    println!("✅ TextBuilder PDF created: {}", output_path);
}

#[test]
fn test_custom_fonts_pdf() {
    ensure_output_dir();

    let mut doc = Document::with_version("1.7");
    let mut font_manager = FontManager::new();

    // Setup document structure
    let pages_id = doc.add_object(dictionary! {
        "Type" => "Pages",
        "Count" => 1,
    });

    // Load custom fonts from test assets
    let inter_font = Font::from_file("tests/assets/fonts/Inter-Variable.ttf")
        .expect("Failed to load Inter font");
    let jetbrains_font = Font::from_file("tests/assets/fonts/JetBrainsMono-Variable.ttf")
        .expect("Failed to load JetBrains Mono font");
    let roboto_font = Font::from_file("tests/assets/fonts/RobotoMono-Variable.ttf")
        .expect("Failed to load Roboto Mono font");

    let (_, f1) = font_manager
        .embed_font(&mut doc, inter_font.clone())
        .unwrap();
    let (_, f2) = font_manager
        .embed_font(&mut doc, jetbrains_font.clone())
        .unwrap();
    let (_, f3) = font_manager
        .embed_font(&mut doc, roboto_font.clone())
        .unwrap();

    // Build content
    let mut operations = Vec::new();

    // Helper to add encoded text
    let add_text = |ops: &mut Vec<Operation>,
                    font: &Font,
                    font_name: &str,
                    text: &str,
                    x: f32,
                    y: f32,
                    size: f32| {
        ops.push(Ops::begin_text());
        ops.push(Ops::set_font(font_name, size));
        ops.push(Ops::position(x, y));

        if font.needs_utf16_encoding() {
            let encoded = font.encode_text(text);
            ops.push(Ops::show_encoded(encoded));
        } else {
            ops.push(Ops::show(text));
        }

        ops.push(Ops::end_text());
    };

    // Title
    operations.push(Ops::set_fill_color_rgb(0.0, 0.0, 0.0));
    add_text(
        &mut operations,
        &inter_font,
        &f1,
        "Custom Embedded Fonts Test",
        50.0,
        750.0,
        24.0,
    );

    // Inter font
    add_text(
        &mut operations,
        &inter_font,
        &f1,
        "Inter Variable: The quick brown fox jumps over the lazy dog",
        50.0,
        700.0,
        16.0,
    );
    add_text(
        &mut operations,
        &inter_font,
        &f1,
        "A raposa castanha rápida pula sobre o cão preguiçoso",
        50.0,
        680.0,
        14.0,
    );

    // JetBrains Mono font
    add_text(
        &mut operations,
        &jetbrains_font,
        &f2,
        "JetBrains Mono: The quick brown fox jumps over the lazy dog",
        50.0,
        650.0,
        14.0,
    );
    add_text(
        &mut operations,
        &jetbrains_font,
        &f2,
        "A raposa castanha rápida pula sobre o cão preguiçoso",
        50.0,
        630.0,
        12.0,
    );

    // Roboto Mono font
    add_text(
        &mut operations,
        &roboto_font,
        &f3,
        "Roboto Mono: The quick brown fox jumps over the lazy dog",
        50.0,
        600.0,
        14.0,
    );
    add_text(
        &mut operations,
        &roboto_font,
        &f3,
        "A raposa castanha rápida pula sobre o cão preguiçoso",
        50.0,
        580.0,
        12.0,
    );

    // Different sizes with Inter
    add_text(
        &mut operations,
        &inter_font,
        &f1,
        "10pt Inter: Small text with custom font",
        50.0,
        540.0,
        10.0,
    );
    add_text(
        &mut operations,
        &inter_font,
        &f1,
        "14pt Inter: Normal text with custom font",
        50.0,
        520.0,
        14.0,
    );
    add_text(
        &mut operations,
        &inter_font,
        &f1,
        "20pt Inter: Large text with custom font",
        50.0,
        490.0,
        20.0,
    );

    // Colors with JetBrains Mono
    operations.push(Ops::set_fill_color_rgb(1.0, 0.0, 0.0));
    add_text(
        &mut operations,
        &jetbrains_font,
        &f2,
        "Red JetBrains Mono text",
        50.0,
        440.0,
        16.0,
    );

    operations.push(Ops::set_fill_color_rgb(0.0, 0.0, 1.0));
    add_text(
        &mut operations,
        &jetbrains_font,
        &f2,
        "Blue JetBrains Mono text",
        50.0,
        415.0,
        16.0,
    );

    // Special characters and symbols
    operations.push(Ops::set_fill_color_rgb(0.0, 0.0, 0.0));
    add_text(
        &mut operations,
        &inter_font,
        &f1,
        "Inter: Symbols: αβγδεζηθικλμνξοπρστυφχψω",
        50.0,
        370.0,
        14.0,
    );

    add_text(
        &mut operations,
        &jetbrains_font,
        &f2,
        "JetBrains: Code: fn main() { println!(\"Hello, World!\"); }",
        50.0,
        345.0,
        14.0,
    );

    // Create content stream
    let content = Content { operations };
    let content_stream = Stream::new(dictionary! {}, content.encode().unwrap());
    let content_id = doc.add_object(content_stream);

    // Create resources with fonts
    let mut resources = dictionary! {};
    for (_font, font_id, resource_name) in font_manager.fonts() {
        font_manager.add_to_resources(&mut resources, *font_id, resource_name);
    }

    // Create page
    let page_id = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => content_id,
        "Resources" => resources,
    });

    // Update pages
    if let Ok(Object::Dictionary(ref mut pages)) = doc.get_object_mut(pages_id) {
        pages.set("Kids", vec![Object::Reference(page_id)]);
    }

    // Create catalog
    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => Object::Reference(pages_id),
    });

    doc.trailer.set("Root", Object::Reference(catalog_id));

    // Save
    let output_path = format!("{}/fonts_custom_test.pdf", TEST_OUTPUT_DIR);
    doc.save(&output_path).expect("Failed to save PDF");

    assert!(Path::new(&output_path).exists());
    println!("✅ Custom fonts PDF created: {}", output_path);
}

#[test]
fn test_postscript_name_sanitization() {
    let font = Font::from_file("tests/assets/fonts/Inter-Variable.ttf")
        .expect("Failed to load Inter font");

    let postscript = &font.metadata.postscript_name;
    assert!(!postscript.is_empty());
    assert!(postscript.chars().all(|ch| !ch.is_whitespace()));
}

#[test]
fn test_embed_font_uses_postscript_name() {
    let mut doc = Document::with_version("1.7");
    let mut font_manager = FontManager::new();

    let font = Font::from_file("tests/assets/fonts/Inter-Variable.ttf")
        .expect("Failed to load Inter font");

    let (type0_id, _resource_name) = font_manager
        .embed_font(&mut doc, font.clone())
        .expect("Failed to embed font");

    let type0 = doc
        .get_object(type0_id)
        .expect("Embedded Type0 font not found");

    let type0_dict = match type0 {
        Object::Dictionary(dict) => dict,
        _ => panic!("Type0 font object is not a dictionary"),
    };

    let base_font = type0_dict
        .get(b"BaseFont")
        .expect("Type0 font BaseFont missing");
    match base_font {
        Object::Name(name_bytes) => {
            let name = std::str::from_utf8(name_bytes).expect("Invalid BaseFont name");
            assert_eq!(name, font.metadata.postscript_name.as_str());
        }
        other => panic!("Expected BaseFont name object, got {:?}", other),
    }

    let descendant_fonts = type0_dict
        .get(b"DescendantFonts")
        .expect("DescendantFonts missing");
    let cid_font_id = match descendant_fonts {
        Object::Array(entries) => match entries.first() {
            Some(Object::Reference(id)) => *id,
            _ => panic!("DescendantFonts does not contain a reference"),
        },
        _ => panic!("DescendantFonts is not an array"),
    };

    let cid_font = doc
        .get_object(cid_font_id)
        .expect("CIDFont dictionary missing");
    let cid_font_dict = match cid_font {
        Object::Dictionary(dict) => dict,
        _ => panic!("CIDFont object is not a dictionary"),
    };

    let cid_base_font = cid_font_dict
        .get(b"BaseFont")
        .expect("CIDFont BaseFont missing");
    match cid_base_font {
        Object::Name(name_bytes) => {
            let name = std::str::from_utf8(name_bytes).expect("Invalid CID BaseFont name");
            assert_eq!(name, font.metadata.postscript_name.as_str());
        }
        _ => panic!("CIDFont BaseFont is not a name"),
    }

    let descriptor_ref = cid_font_dict
        .get(b"FontDescriptor")
        .expect("FontDescriptor reference missing");
    let descriptor_id = match descriptor_ref {
        Object::Reference(id) => *id,
        _ => panic!("FontDescriptor is not a reference"),
    };

    let descriptor = doc
        .get_object(descriptor_id)
        .expect("FontDescriptor dictionary missing");
    let descriptor_dict = match descriptor {
        Object::Dictionary(dict) => dict,
        _ => panic!("FontDescriptor object is not a dictionary"),
    };

    let font_name = descriptor_dict
        .get(b"FontName")
        .expect("FontDescriptor FontName missing");
    match font_name {
        Object::Name(name_bytes) => {
            let name = std::str::from_utf8(name_bytes).expect("Invalid FontDescriptor FontName");
            assert_eq!(name, font.metadata.postscript_name.as_str());
        }
        _ => panic!("FontDescriptor FontName is not a name"),
    }
}

#[test]
fn test_font_metadata_extraction() {
    // Test with a standard font if available
    if let Ok(font_data) = std::fs::read("/System/Library/Fonts/Helvetica.ttc")
        .or_else(|_| std::fs::read("C:\\Windows\\Fonts\\arial.ttf"))
        .or_else(|_| std::fs::read("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"))
    {
        match Font::from_bytes(font_data, None) {
            Ok(font) => {
                println!("✅ Font family: {}", font.family());
                println!("   Weight: {}", font.metadata.weight);
                println!("   Italic: {}", font.metadata.italic);
                assert!(!font.family().is_empty());
                assert!(font.family() != "Unknown Font");
            }
            Err(e) => println!("⚠️  Font parsing skipped: {}", e),
        }
    } else {
        println!("⚠️  No system fonts found for testing");
    }
}

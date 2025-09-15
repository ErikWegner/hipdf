//! # hipdf
//!
//! A high-level PDF manipulation library built on lopdf, focusing on ease of use
//! and powerful abstractions for common PDF operations.
//!
//! ## Features
//!
//! - **OCG (Optional Content Groups) Support**: Easy creation and management of PDF layers
//! - **Layer Management**: High-level API for organizing content into toggleable layers
//! - **Content Building**: Fluent API for building layered PDF content
//! - **Type Safety**: Strongly typed interfaces with compile-time guarantees
//! - **WASM Support**: Automatic configuration for WebAssembly targets
//!
//! ## Example
//!
//! ```rust
//! use hipdf::ocg::{OCGManager, Layer, LayerContentBuilder, LayerOperations as Ops};
//! use lopdf::{Document, Object};
//!
//! // Create a new PDF with layers
//! let mut doc = Document::with_version("1.5");
//! let mut ocg_manager = OCGManager::with_config(Default::default());
//!
//! // Add layers
//! ocg_manager.add_layer(Layer::new("Background", true));
//! ocg_manager.add_layer(Layer::new("Main Content", true));
//!
//! // Initialize layers in document
//! ocg_manager.initialize(&mut doc);
//! ```
//!
//! ## WASM Support
//!
//! This library automatically configures itself for WASM targets.
//! ```rust
//! #[cfg(target_arch = "wasm32")]
//! hipdf::init_wasm();
//! ```
//!
//! ## Modules
//!
//! - [`ocg`] - Optional Content Groups (layers) functionality
//! - [`hatching`] - Hatching and pattern support for PDF documents
//! - [`embed_pdf`] - Embedding existing PDF documents
//! - [`blocks`] - Block content management
//! - [`images`] - Image embedding and manipulation

pub mod embed_pdf;
pub mod hatching;
pub mod ocg;
pub mod blocks;
pub mod images;

pub use lopdf;

// WASM-specific configuration
#[cfg(target_arch = "wasm32")]
pub fn init_wasm() {
    // Initialize WASM-specific features
    // This function can be called by users to ensure proper WASM setup
}

// Common type aliases and utilities
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub type Error = Box<dyn std::error::Error>;

#[cfg(test)]
mod tests {
    // Tests are in separate integration test files
}

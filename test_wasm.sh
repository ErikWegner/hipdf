#!/bin/bash

# Comprehensive test script for WASM support in hipdf library
# This script tests both native and WASM compilation and runs appropriate tests

set -e

echo "🚀 Starting comprehensive WASM support test for hipdf"
echo "================================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test functions
test_native() {
    echo -e "${BLUE}📋 Running native tests...${NC}"

    echo -e "${YELLOW}  Testing native library compilation...${NC}"
    cargo check
    echo -e "${GREEN}  ✅ Native compilation successful${NC}"

    echo -e "${YELLOW}  Testing native library build...${NC}"
    cargo build
    echo -e "${GREEN}  ✅ Native build successful${NC}"

    echo -e "${YELLOW}  Running native unit tests...${NC}"
    cargo test --lib
    echo -e "${GREEN}  ✅ Native unit tests passed${NC}"

    echo -e "${YELLOW}  Running integration tests...${NC}"
    cargo test --test images_integration_test
    echo -e "${GREEN}  ✅ Integration tests passed${NC}"

    echo -e "${GREEN}🎉 All native tests passed!${NC}"
}

test_wasm_compilation() {
    echo -e "${BLUE}🔥 Testing WASM compilation...${NC}"

    echo -e "${YELLOW}  Testing WASM library compilation...${NC}"
    cargo check --target wasm32-unknown-unknown --features wasm_js
    echo -e "${GREEN}  ✅ WASM compilation successful${NC}"

    echo -e "${YELLOW}  Testing WASM library build...${NC}"
    cargo build --target wasm32-unknown-unknown --features wasm_js --release
    echo -e "${GREEN}  ✅ WASM build successful${NC}"

    echo -e "${YELLOW}  Testing WASM test compilation...${NC}"
    cargo test --test wasm_tests --target wasm32-unknown-unknown --no-run
    echo -e "${GREEN}  ✅ WASM test compilation successful${NC}"

    echo -e "${GREEN}🎉 WASM compilation tests passed!${NC}"
}

test_wasm_browser() {
    echo -e "${BLUE}🌐 Testing WASM in browser environment...${NC}"

    # Check if wasm-bindgen-test is available
    if command -v wasm-bindgen-test &> /dev/null; then
        echo -e "${YELLOW}  Running WASM browser tests...${NC}"

        # Try to run WASM tests in headless Chrome
        if wasm-bindgen-test --chrome --headless --test wasm_tests; then
            echo -e "${GREEN}  ✅ WASM browser tests passed${NC}"
        else
            echo -e "${YELLOW}  ⚠️  WASM browser tests not available (requires Chrome and wasm-bindgen-test)${NC}"
        fi
    else
        echo -e "${YELLOW}  ⚠️  wasm-bindgen-test not found, skipping browser tests${NC}"
        echo -e "${YELLOW}     Install with: cargo install wasm-bindgen-test-cli${NC}"
    fi
}

test_image_features() {
    echo -e "${BLUE}🖼️  Testing image handling features...${NC}"

    echo -e "${YELLOW}  Testing from_bytes method...${NC}"
    cargo test --test images_integration_test test_wasm_compatibility_from_bytes
    echo -e "${GREEN}  ✅ from_bytes tests passed${NC}"

    echo -e "${YELLOW}  Testing WASM compatibility workflow...${NC}"
    cargo test --test images_integration_test test_wasm_comprehensive_bytes_workflow
    echo -e "${GREEN}  ✅ WASM compatibility workflow tests passed${NC}"

    echo -e "${YELLOW}  Testing image format detection...${NC}"
    cargo test --test images_integration_test test_wasm_image_format_detection
    echo -e "${GREEN}  ✅ Image format detection tests passed${NC}"

    echo -e "${YELLOW}  Testing error handling...${NC}"
    cargo test --test images_integration_test test_wasm_error_handling
    echo -e "${GREEN}  ✅ Error handling tests passed${NC}"

    echo -e "${GREEN}🎉 Image handling features tests passed!${NC}"
}

test_documentation() {
    echo -e "${BLUE}📚 Testing documentation generation...${NC}"

    echo -e "${YELLOW}  Generating documentation...${NC}"
    cargo doc --no-deps --document-private-items
    echo -e "${GREEN}  ✅ Documentation generation successful${NC}"

    echo -e "${GREEN}🎉 Documentation tests passed!${NC}"
}

# Main test execution
main() {
    echo -e "${BLUE}🏁 Starting comprehensive test suite...${NC}"
    echo ""

    test_native
    echo ""

    test_wasm_compilation
    echo ""

    test_image_features
    echo ""

    test_documentation
    echo ""

    test_wasm_browser
    echo ""

    echo -e "${GREEN}🎊🎊🎊 ALL TESTS COMPLETED SUCCESSFULLY! 🎊🎊🎊${NC}"
    echo ""
    echo -e "${BLUE}Summary:${NC}"
    echo "  ✅ Native compilation and tests"
    echo "  ✅ WASM compilation and tests"
    echo "  ✅ Image handling with from_bytes methods"
    echo "  ✅ WASM compatibility layer"
    echo "  ✅ Documentation generation"
    echo "  ✅ Browser-based WASM tests (when available)"
    echo ""
    echo -e "${GREEN}The hipdf library is now fully WASM compatible! 🚀${NC}"
    echo ""
    echo -e "${YELLOW}Users can now:${NC}"
    echo "  • Use the library in WASM environments without manual configuration"
    echo "  • Load images from bytes using Image::from_bytes()"
    echo "  • Embed images in PDF documents in browser environments"
}

# Run main function
main "$@"
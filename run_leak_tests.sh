#!/bin/bash
# Run memory leak tests for screencapturekit-rs

set -e

echo "🧪 ScreenCaptureKit Memory Leak Tests"
echo "======================================"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo -e "${RED}Error: Leak tests require macOS (uses 'leaks' command)${NC}"
    exit 1
fi

# Check if 'leaks' command is available
if ! command -v leaks &> /dev/null; then
    echo -e "${RED}Error: 'leaks' command not found${NC}"
    echo "This command is part of Xcode Command Line Tools"
    exit 1
fi

echo -e "${BLUE}ℹ️  Note: These tests require Screen Recording permission${NC}"
echo ""

# Parse arguments
FEATURE_FLAGS=""
TEST_FILTER=""
VERBOSE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --all-features)
            FEATURE_FLAGS="--all-features"
            shift
            ;;
        --async)
            FEATURE_FLAGS="--features async"
            shift
            ;;
        --macos-15)
            FEATURE_FLAGS="--features macos_15_0"
            shift
            ;;
        --macos-14)
            FEATURE_FLAGS="--features macos_14_0"
            shift
            ;;
        --test)
            TEST_FILTER="$2"
            shift 2
            ;;
        --verbose)
            VERBOSE="--nocapture"
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --all-features    Run with all features enabled"
            echo "  --async           Run with async feature"
            echo "  --macos-15        Run with macOS 15.0 features"
            echo "  --macos-14        Run with macOS 14.0 features"
            echo "  --test <name>     Run specific test (e.g., test_display_clone_no_leak)"
            echo "  --verbose         Show detailed test output"
            echo "  --help            Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                                  # Run all leak tests"
            echo "  $0 --async                          # Run with async feature"
            echo "  $0 --test test_display_clone_no_leak  # Run specific test"
            echo "  $0 --all-features --verbose         # Run all with verbose output"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo -e "${YELLOW}Building tests...${NC}"
if [ -n "$FEATURE_FLAGS" ]; then
    cargo test --test refcount_leak_test $FEATURE_FLAGS --no-run
else
    cargo test --test refcount_leak_test --no-run
fi

echo ""
echo -e "${GREEN}✓ Build complete${NC}"
echo ""

# Run the tests
echo -e "${YELLOW}Running memory leak tests...${NC}"
echo ""

if [ -n "$TEST_FILTER" ]; then
    echo -e "${BLUE}Running test: $TEST_FILTER${NC}"
    echo ""
    cargo test --test refcount_leak_test $FEATURE_FLAGS "$TEST_FILTER" -- --test-threads=1 $VERBOSE
else
    echo -e "${BLUE}Running all leak tests${NC}"
    echo ""
    cargo test --test refcount_leak_test $FEATURE_FLAGS -- --test-threads=1 $VERBOSE
fi

EXIT_CODE=$?

echo ""
if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✅ All leak tests passed!${NC}"
else
    echo -e "${RED}❌ Some tests failed${NC}"
    exit $EXIT_CODE
fi

# Summary
echo ""
echo "======================================"
echo -e "${GREEN}Memory Leak Test Summary${NC}"
echo "======================================"
echo ""
echo "Tests completed successfully!"
echo ""
echo "To run specific test suites:"
echo "  • Basic tests:       ./run_leak_tests.sh"
echo "  • With async:        ./run_leak_tests.sh --async"
echo "  • All features:      ./run_leak_tests.sh --all-features"
echo "  • Specific test:     ./run_leak_tests.sh --test test_name"
echo ""
echo "For debugging leaks:"
echo "  • Instruments:       Open Xcode Instruments and profile 'Leaks'"
echo "  • Manual check:      leaks <PID>"
echo "  • Verbose output:    ./run_leak_tests.sh --verbose"
echo ""

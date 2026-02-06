#!/bin/bash
# Local CI test script - runs the same checks as GitHub Actions

set -e

BOLD='\033[1m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

print_step() {
    echo -e "\n${BOLD}${GREEN}==>${NC} ${BOLD}$1${NC}"
}

print_warning() {
    echo -e "${YELLOW}Warning:${NC} $1"
}

print_error() {
    echo -e "${RED}Error:${NC} $1"
}

# Parse arguments
RUN_ALL=true
RUN_FMT=false
RUN_CLIPPY=false
RUN_TEST=false
RUN_BUILD=false
RUN_DOC=false
RUN_AUDIT=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --fmt)
            RUN_ALL=false
            RUN_FMT=true
            shift
            ;;
        --clippy)
            RUN_ALL=false
            RUN_CLIPPY=true
            shift
            ;;
        --test)
            RUN_ALL=false
            RUN_TEST=true
            shift
            ;;
        --build)
            RUN_ALL=false
            RUN_BUILD=true
            shift
            ;;
        --doc)
            RUN_ALL=false
            RUN_DOC=true
            shift
            ;;
        --audit)
            RUN_ALL=false
            RUN_AUDIT=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --fmt      Run format check only"
            echo "  --clippy   Run clippy lint only"
            echo "  --test     Run tests only"
            echo "  --build    Run release build only"
            echo "  --doc      Run documentation build only"
            echo "  --audit    Run security audit only"
            echo "  --help     Show this help message"
            echo ""
            echo "If no options specified, runs all checks."
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Track failures
FAILED=0

# Format check
if [[ "$RUN_ALL" == true || "$RUN_FMT" == true ]]; then
    print_step "Checking code formatting..."
    if cargo fmt --all -- --check; then
        echo -e "${GREEN}Format check passed${NC}"
    else
        print_error "Format check failed. Run 'cargo fmt --all' to fix."
        FAILED=1
    fi
fi

# Clippy
if [[ "$RUN_ALL" == true || "$RUN_CLIPPY" == true ]]; then
    print_step "Running clippy..."
    if cargo clippy --all-targets --all-features -- -D warnings; then
        echo -e "${GREEN}Clippy passed${NC}"
    else
        print_error "Clippy found issues"
        FAILED=1
    fi
fi

# Tests
if [[ "$RUN_ALL" == true || "$RUN_TEST" == true ]]; then
    print_step "Running tests..."
    if cargo test --all --verbose; then
        echo -e "${GREEN}All tests passed${NC}"
    else
        print_error "Tests failed"
        FAILED=1
    fi
fi

# Build release
if [[ "$RUN_ALL" == true || "$RUN_BUILD" == true ]]; then
    print_step "Building release binaries..."
    if cargo build --release --all; then
        echo -e "${GREEN}Release build successful${NC}"
        echo "Binaries:"
        ls -lh target/release/greport target/release/greport-api 2>/dev/null || true
    else
        print_error "Release build failed"
        FAILED=1
    fi
fi

# Documentation
if [[ "$RUN_ALL" == true || "$RUN_DOC" == true ]]; then
    print_step "Building documentation..."
    if RUSTDOCFLAGS="-D warnings" cargo doc --all --no-deps; then
        echo -e "${GREEN}Documentation build successful${NC}"
    else
        print_error "Documentation build failed"
        FAILED=1
    fi
fi

# Security audit
if [[ "$RUN_ALL" == true || "$RUN_AUDIT" == true ]]; then
    print_step "Running security audit..."
    if command -v cargo-audit &> /dev/null; then
        if cargo audit; then
            echo -e "${GREEN}Security audit passed${NC}"
        else
            print_warning "Security audit found issues (non-blocking)"
        fi
    else
        print_warning "cargo-audit not installed. Run 'cargo install cargo-audit' to enable."
    fi
fi

# Summary
echo ""
echo "========================================"
if [[ $FAILED -eq 0 ]]; then
    echo -e "${GREEN}${BOLD}All CI checks passed!${NC}"
    exit 0
else
    echo -e "${RED}${BOLD}Some CI checks failed${NC}"
    exit 1
fi

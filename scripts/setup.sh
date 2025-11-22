#!/usr/bin/env bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║         Typthon Development Environment Setup            ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo -e "${RED}✗ This script is currently designed for macOS${NC}"
    echo -e "${YELLOW}  For other platforms, install dependencies manually${NC}"
    exit 1
fi

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install or update via brew
brew_install_or_update() {
    local package=$1
    local tap=$2

    if [[ -n "$tap" ]]; then
        echo -e "${BLUE}→ Adding brew tap: $tap${NC}"
        brew tap "$tap" 2>/dev/null || true
    fi

    if brew list "$package" &>/dev/null; then
        echo -e "${GREEN}✓ $package already installed${NC}"
    else
        echo -e "${YELLOW}→ Installing $package${NC}"
        brew install "$package"
        echo -e "${GREEN}✓ $package installed${NC}"
    fi
}

# 1. Check/Install Homebrew
echo -e "\n${BLUE}[1/8] Checking Homebrew...${NC}"
if ! command_exists brew; then
    echo -e "${YELLOW}→ Installing Homebrew${NC}"
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    echo -e "${GREEN}✓ Homebrew installed${NC}"
else
    echo -e "${GREEN}✓ Homebrew found${NC}"
fi

# 2. Install Builder
echo -e "\n${BLUE}[2/8] Installing Builder...${NC}"
if ! command_exists builder; then
    echo -e "${YELLOW}→ Adding griffincancode/builder tap${NC}"
    brew tap griffincancode/builder
    echo -e "${YELLOW}→ Installing builder${NC}"
    brew install builder
    echo -e "${GREEN}✓ Builder installed${NC}"
else
    echo -e "${GREEN}✓ Builder found (version: $(builder --version 2>/dev/null | head -n1 || echo 'unknown'))${NC}"
fi

# 3. Install Rust and Cargo
echo -e "\n${BLUE}[3/8] Installing Rust toolchain...${NC}"
if ! command_exists rustc; then
    echo -e "${YELLOW}→ Installing Rust via rustup${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    source "$HOME/.cargo/env"
    echo -e "${GREEN}✓ Rust installed${NC}"
else
    echo -e "${GREEN}✓ Rust found (version: $(rustc --version))${NC}"
fi

# Ensure cargo is in PATH
if ! command_exists cargo; then
    export PATH="$HOME/.cargo/bin:$PATH"
fi

# Install maturin for Python bindings
if ! command_exists maturin; then
    echo -e "${YELLOW}→ Installing maturin${NC}"
    cargo install maturin
    echo -e "${GREEN}✓ maturin installed${NC}"
else
    echo -e "${GREEN}✓ maturin found${NC}"
fi

# 4. Install Python
echo -e "\n${BLUE}[4/8] Installing Python...${NC}"
brew_install_or_update "python@3.10"

# Ensure python3 is available
if ! command_exists python3; then
    echo -e "${RED}✗ python3 not found in PATH${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Python found (version: $(python3 --version))${NC}"

# 5. Install Python development tools
echo -e "\n${BLUE}[5/8] Installing Python development tools...${NC}"
python3 -m pip install --upgrade pip setuptools wheel

# Install required Python packages
PYTHON_PACKAGES=(
    "pytest"
    "pytest-benchmark"
    "mypy"
    "ruff"
    "maturin"
)

for package in "${PYTHON_PACKAGES[@]}"; do
    if python3 -m pip show "$package" &>/dev/null; then
        echo -e "${GREEN}✓ $package already installed${NC}"
    else
        echo -e "${YELLOW}→ Installing $package${NC}"
        python3 -m pip install "$package"
        echo -e "${GREEN}✓ $package installed${NC}"
    fi
done

# 6. Install Go
echo -e "\n${BLUE}[6/8] Installing Go...${NC}"
if ! command_exists go; then
    brew_install_or_update "go"
    echo -e "${GREEN}✓ Go installed${NC}"
else
    echo -e "${GREEN}✓ Go found (version: $(go version))${NC}"
fi

# 7. Install C/C++ toolchain
echo -e "\n${BLUE}[7/8] Checking C/C++ toolchain...${NC}"
if ! command_exists gcc; then
    echo -e "${YELLOW}→ Installing gcc${NC}"
    brew_install_or_update "gcc"
    echo -e "${GREEN}✓ gcc installed${NC}"
else
    echo -e "${GREEN}✓ gcc found (version: $(gcc --version | head -n1))${NC}"
fi

if ! command_exists clang; then
    echo -e "${YELLOW}→ Installing llvm${NC}"
    brew_install_or_update "llvm"
    echo -e "${GREEN}✓ llvm/clang installed${NC}"
else
    echo -e "${GREEN}✓ clang found (version: $(clang --version | head -n1))${NC}"
fi

# 8. Install additional build tools
echo -e "\n${BLUE}[8/8] Installing additional build tools...${NC}"
brew_install_or_update "cmake"
brew_install_or_update "make"

# Verify all critical tools
echo -e "\n${BLUE}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                  Verification Summary                     ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""

MISSING_TOOLS=()

check_tool() {
    local tool=$1
    local name=$2
    if command_exists "$tool"; then
        echo -e "${GREEN}✓ $name${NC}"
        return 0
    else
        echo -e "${RED}✗ $name${NC}"
        MISSING_TOOLS+=("$name")
        return 1
    fi
}

check_tool "builder" "Builder"
check_tool "rustc" "Rust"
check_tool "cargo" "Cargo"
check_tool "maturin" "Maturin"
check_tool "python3" "Python 3"
check_tool "pytest" "pytest"
check_tool "mypy" "mypy"
check_tool "ruff" "ruff"
check_tool "go" "Go"
check_tool "gcc" "GCC"
check_tool "clang" "Clang"
check_tool "cmake" "CMake"
check_tool "make" "Make"

echo ""

if [ ${#MISSING_TOOLS[@]} -eq 0 ]; then
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║             ✓ Setup Complete! All tools ready             ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${BLUE}Next steps:${NC}"
    echo -e "  1. Run ${YELLOW}make build${NC} to build all targets"
    echo -e "  2. Run ${YELLOW}make test${NC} to run tests"
    echo -e "  3. Run ${YELLOW}builder graph${NC} to visualize dependencies"
    echo ""
else
    echo -e "${RED}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║        ✗ Setup incomplete - missing tools detected        ║${NC}"
    echo -e "${RED}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${RED}Missing tools: ${MISSING_TOOLS[*]}${NC}"
    echo -e "${YELLOW}Please install missing tools manually or re-run this script${NC}"
    exit 1
fi


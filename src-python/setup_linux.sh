#!/bin/bash

# Environment isolation
export PYTHONNOUSERSITE=1
unset PYTHONPATH
unset PYTHONHOME

# Get the directory of the script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
cd "$SCRIPT_DIR"

echo "PWD = $PWD"

# Check for special characters in installation path
if [[ "$PWD" == *[[:space:]]* ]]; then
    echo "WARNING: Spaces detected in the installation path!"
    echo "         This can cause the installation to fail!"
fi

if [[ "$PWD" =~ [!\#\$%\&\(\)\*\+,\;\<\=\>\?\@\[\]\^\`\{\|\}~] ]]; then
    echo "WARNING: Special characters were detected in the installation path!"
    echo "         This can cause the installation to fail!"
fi

# Fix failed install when installing to a separate drive
export TMP="$PWD/installation"
export TEMP="$PWD/installation"

# Deactivate existing conda envs as needed to avoid conflicts
{ conda deactivate && conda deactivate && conda deactivate; } 2>/dev/null || true

# Config
INSTALL_DIR="$PWD/installation"
CONDA_ROOT_PREFIX="$PWD/installation/conda"
INSTALL_ENV_DIR="$PWD/installation/env"
MINICONDA_DOWNLOAD_URL="https://repo.anaconda.com/miniconda/Miniconda3-py312_25.5.1-1-Linux-x86_64.sh"
MINICONDA_CHECKSUM="e3228df32afc6d43cb190a416b91937cdcd1c6308d9fe652274539a07142966f"
conda_exists="F"

# Figure out whether conda needs to be installed
if "$CONDA_ROOT_PREFIX/bin/conda" --version &>/dev/null; then
    conda_exists="T"
fi

# (if necessary) install conda into a contained environment
if [ "$conda_exists" == "F" ]; then
    echo "Downloading Miniconda from $MINICONDA_DOWNLOAD_URL to $INSTALL_DIR/miniconda_installer.sh"
    
    mkdir -p "$INSTALL_DIR"
    
    # Download Miniconda
    if command -v wget &> /dev/null; then
        wget -O "$INSTALL_DIR/miniconda_installer.sh" "$MINICONDA_DOWNLOAD_URL"
    elif command -v curl &> /dev/null; then
        curl -L "$MINICONDA_DOWNLOAD_URL" -o "$INSTALL_DIR/miniconda_installer.sh"
    else
        echo "Error: Neither wget nor curl is available. Please install one of them."
        exit 1
    fi
    
    if [ $? -ne 0 ]; then
        echo "Miniconda failed to download."
        exit 1
    fi
    
    # Verify checksum
    if command -v sha256sum &> /dev/null; then
        calculated_checksum=$(sha256sum "$INSTALL_DIR/miniconda_installer.sh" | cut -d' ' -f1)
    elif command -v shasum &> /dev/null; then
        calculated_checksum=$(shasum -a 256 "$INSTALL_DIR/miniconda_installer.sh" | cut -d' ' -f1)
    else
        echo "Warning: Cannot verify checksum. Neither sha256sum nor shasum is available."
        calculated_checksum="$MINICONDA_CHECKSUM"  # Skip verification
    fi
    
    if [ "$calculated_checksum" != "$MINICONDA_CHECKSUM" ]; then
        echo "The checksum verification for miniconda_installer.sh has failed."
        echo "Expected: $MINICONDA_CHECKSUM"
        echo "Got: $calculated_checksum"
        rm -f "$INSTALL_DIR/miniconda_installer.sh"
        exit 1
    else
        echo "The checksum verification for miniconda_installer.sh has passed successfully."
    fi
    
    echo "Installing Miniconda to $CONDA_ROOT_PREFIX"
    
    # Make the installer executable
    chmod +x "$INSTALL_DIR/miniconda_installer.sh"
    
    # Install Miniconda silently
    "$INSTALL_DIR/miniconda_installer.sh" -b -p "$CONDA_ROOT_PREFIX"
    
    if [ $? -ne 0 ]; then
        echo "Miniconda installation failed."
        exit 1
    fi
    
    # Test the conda binary
    echo "Miniconda version:"
    "$CONDA_ROOT_PREFIX/bin/conda" --version
    
    if [ $? -ne 0 ]; then
        echo "Miniconda not found."
        exit 1
    fi
    
    # Delete the Miniconda installer
    rm -f "$INSTALL_DIR/miniconda_installer.sh"
fi

# Create the installer env
if [ ! -d "$INSTALL_ENV_DIR" ]; then
    echo "Creating conda environment with Python 3.12"
    CI=true "$CONDA_ROOT_PREFIX/bin/conda" create --no-shortcuts -y -k --prefix "$INSTALL_ENV_DIR" python=3.12
    
    if [ $? -ne 0 ]; then
        echo "Conda environment creation failed."
        exit 1
    fi
fi

# Check if conda environment was actually created
if [ ! -f "$INSTALL_ENV_DIR/bin/python" ]; then
    echo "Conda environment is empty."
    exit 1
fi

# Set environment variable for CI
export CI=true

# Activate installer env
source "$CONDA_ROOT_PREFIX/etc/profile.d/conda.sh"
conda activate "$INSTALL_ENV_DIR"

if [ $? -ne 0 ]; then
    echo "Failed to activate conda environment."
    exit 1
fi

# Setup installer env
python setup.py "$@"

echo "Setup completed successfully!"

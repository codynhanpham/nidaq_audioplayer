#!/bin/bash

# Set script title for terminals that support it
echo -e "\033]0;NI-DAQmx Audio Player WS Server\007"

# Get the directory of the script and change to it
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
cd "$SCRIPT_DIR"

# Check for spaces in path (similar to Windows script)
if [[ "$PWD" == *[[:space:]]* ]]; then
    echo "This script relies on Miniconda which can not be silently installed under a path with spaces."
    exit 1
fi

# Fix failed install when installing to a separate drive
export TMP="$PWD/installation"
export TEMP="$PWD/installation"

# Deactivate existing conda envs as needed to avoid conflicts
{ conda deactivate && conda deactivate && conda deactivate; } 2>/dev/null || true

echo "Starting the Python WebSocket server..."
echo "This will start the server in just a moment :>"
echo

# Config
CONDA_ROOT_PREFIX="$PWD/installation/conda"
INSTALL_ENV_DIR="$PWD/installation/env"

# Environment isolation
export PYTHONNOUSERSITE=1
unset PYTHONPATH
unset PYTHONHOME
# export CUDA_PATH="$INSTALL_ENV_DIR"
# export CUDA_HOME="$CUDA_PATH"

# Activate installer env
echo "[1/2] Activating conda environment \"$INSTALL_ENV_DIR\"..."
if ! source "$CONDA_ROOT_PREFIX/etc/profile.d/conda.sh"; then
    echo
    echo "Miniconda hook not found."
    exit 1
fi

if ! conda activate "$INSTALL_ENV_DIR"; then
    echo
    echo "Failed to activate conda environment."
    exit 1
fi

# Start the Python WebSocket server
echo "[2/2] Booting up the WebSocket server..."
echo
if ! python src/main.py "$@"; then
    echo
    echo "Failed to start the Python WebSocket server."
    exit 1
fi

echo "WebSocket server started successfully"

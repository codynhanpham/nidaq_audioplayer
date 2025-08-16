#!/bin/bash

# Set script title for terminals that support it
echo -e "\033]0;NI-DAQmx Audio Player CMD\007"

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

echo "[2/2] Starting interactive shell with conda environment activated..."
echo "Type 'exit' to close this shell and return to your original environment."
echo

# Detect the user's preferred shell and start appropriate interactive session
if [[ -n "$FISH_VERSION" ]] || [[ "$SHELL" == *"fish"* ]]; then
    # For fish shell users
    exec fish -c "
        source '$CONDA_ROOT_PREFIX/etc/fish/conf.d/conda.fish' 2>/dev/null || true
        conda activate '$INSTALL_ENV_DIR'
        fish
    "
else
    # Start an interactive bash shell with the conda environment
    exec bash --rcfile <(echo "source ~/.bashrc 2>/dev/null || true; PS1='(env) \u@\h:\w$ '")
fi


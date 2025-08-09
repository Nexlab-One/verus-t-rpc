#!/bin/bash
# Verus Bootstrap Script for Docker
# Downloads blockchain data to speed up initial sync

set -eu

VERUS_DATA_DIR="${VERUS_DATA_DIR:-/home/verus/.komodo/VRSC}"
BOOTSTRAP_URL="https://bootstrap.verus.io"
BOOTSTRAP_ARCHIVE="VRSC-bootstrap.tar.gz"

echo "Verus Bootstrap Script"
echo "======================"

# Create data directory if it doesn't exist
if [ ! -d "$VERUS_DATA_DIR" ]; then
    echo "Creating Verus data directory: $VERUS_DATA_DIR"
    mkdir -p "$VERUS_DATA_DIR"
fi

# Check if bootstrap data already exists
if [ -f "$VERUS_DATA_DIR/blocks/blk00000.dat" ]; then
    echo "Bootstrap data already exists. Skipping download."
    exit 0
fi

echo "Downloading Verus bootstrap data..."
echo "This may take several minutes depending on your connection."

# Download bootstrap archive
if command -v wget >/dev/null 2>&1; then
    wget --progress=dot:giga \
         --output-document="/tmp/$BOOTSTRAP_ARCHIVE" \
         --continue \
         --retry-connrefused --waitretry=3 --timeout=30 \
         "$BOOTSTRAP_URL/$BOOTSTRAP_ARCHIVE"
elif command -v curl >/dev/null 2>&1; then
    curl --output "/tmp/$BOOTSTRAP_ARCHIVE" \
         -# -L -C - \
         "$BOOTSTRAP_URL/$BOOTSTRAP_ARCHIVE"
else
    echo "Error: Neither wget nor curl is available"
    exit 1
fi

# Extract bootstrap data
echo "Extracting bootstrap data..."
cd "$VERUS_DATA_DIR"
tar -xzf "/tmp/$BOOTSTRAP_ARCHIVE"

# Clean up
rm "/tmp/$BOOTSTRAP_ARCHIVE"

echo "Bootstrap data installed successfully!"
echo "Verus daemon will start with pre-synced blockchain data."

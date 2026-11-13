#!/usr/bin/env bash
set -e

if [ -z "$1" ]; then
    echo "Usage: ./translate.sh <path_to_c_directory_or_file> [--out <output_crate_dir>]"
    exit 1
fi

export PATH="$HOME/.cargo/bin:$PATH"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
python3 "$SCRIPT_DIR/scripts/translate_library.py" "$@"


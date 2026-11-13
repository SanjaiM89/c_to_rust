#!/usr/bin/env bash
set -e

if [ -z "$1" ]; then
    echo "Usage: ./convert.sh <git_url_or_local_path> [--out <output_zip_dir>]"
    echo "Example: ./convert.sh https://github.com/benhoyt/inih.git"
    exit 1
fi

export PATH="$HOME/.cargo/bin:$PATH"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
python3 "$SCRIPT_DIR/scripts/convert_repo.py" "$@"


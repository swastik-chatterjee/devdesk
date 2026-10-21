#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

npm --prefix ../.. run tauri -- icon src-tauri/icons/icon.svg --output src-tauri/icons

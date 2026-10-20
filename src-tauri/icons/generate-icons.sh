#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

if command -v magick >/dev/null 2>&1; then
  magick icon.svg -resize 256x256 icon.png
elif command -v convert >/dev/null 2>&1; then
  convert icon.svg -resize 256x256 icon.png
elif command -v rsvg-convert >/dev/null 2>&1; then
  rsvg-convert -w 256 -h 256 -o icon.png icon.svg
else
  echo "No SVG->PNG converter found. Install ImageMagick or librsvg." >&2
  exit 1
fi

echo "Generated icon.png from icon.svg"

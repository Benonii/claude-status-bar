#!/usr/bin/env bash
# Convert a screen recording (webm/mp4) into an optimized GIF for the README,
# using a two-pass palette for clean colour and small size.
#
#   gif-from.sh INPUT [OUTPUT] [MAX_WIDTH] [FPS]
#
# Defaults: OUTPUT=docs/screenshots/demo.gif, MAX_WIDTH=760 (never upscales), FPS=15
set -euo pipefail

in=${1:?usage: gif-from.sh INPUT [OUTPUT] [MAX_WIDTH] [FPS]}
out=${2:-docs/screenshots/demo.gif}
width=${3:-760}
fps=${4:-15}

command -v ffmpeg >/dev/null || { echo "ffmpeg not found" >&2; exit 1; }

pal=$(mktemp --suffix=.png)
trap 'rm -f "$pal"' EXIT

filters="fps=$fps,scale=w='min($width,iw)':h=-1:flags=lanczos"
ffmpeg -y -loglevel error -i "$in" -vf "$filters,palettegen=stats_mode=diff" "$pal"
ffmpeg -y -loglevel error -i "$in" -i "$pal" \
    -lavfi "$filters,paletteuse=dither=bayer:bayer_scale=3" "$out"

echo "Wrote $out ($(du -h "$out" | cut -f1))"

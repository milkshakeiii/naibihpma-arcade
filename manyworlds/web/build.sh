#!/bin/sh
# Build the game to WebAssembly into www/. Optionally make the song for leg 1:
#   ./build.sh                       the game only
#   ./build.sh path/to/remasters.zip  the game, plus www/assets/music/maple.m4a from that zip
set -e
cd "$(dirname "$0")"
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/manyworlds_web.wasm www/
if [ -n "$1" ]; then
  mkdir -p www/assets/music
  python3 -I - "$1" www/assets/music/maple.m4a <<'EOF'
import subprocess, sys, zipfile
src, out = sys.argv[1], sys.argv[2]
with zipfile.ZipFile(src) as z:
    name = next(n for n in z.namelist() if "01 The Corner of Maple St" in n)
    ff = subprocess.Popen(["ffmpeg", "-v", "error", "-y", "-i", "pipe:0", "-c:a", "aac", "-b:a", "160k", out],
                          stdin=subprocess.PIPE)
    with z.open(name) as f:
        while chunk := f.read(1 << 20):
            ff.stdin.write(chunk)
    ff.stdin.close()
    sys.exit(ff.wait())
EOF
fi
ls -l www/manyworlds_web.wasm

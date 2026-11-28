#!/bin/sh
# Build Naibihpma Hold to WebAssembly into www/. Optionally make the soundtrack:
#   ./build.sh                       the game only (it runs silently)
#   ./build.sh path/to/remasters.zip  plus www/assets/music/forest-1..6.m4a, The Enchanted
#                                     Forest of Naibihpma (tracks 26 to 31) in album order
set -e
cd "$(dirname "$0")"
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/hold_web.wasm www/
if [ -n "$1" ]; then
  mkdir -p www/assets/music
  python3 -I - "$1" www/assets/music <<'PY'
import subprocess, sys, zipfile
src, out = sys.argv[1], sys.argv[2]
with zipfile.ZipFile(src) as z:
    for k, num in enumerate(range(26, 32), start=1):
        name = next(n for n in z.namelist() if f" - {num} " in n)
        dest = f"{out}/forest-{k}.m4a"
        ff = subprocess.Popen(["ffmpeg", "-v", "error", "-y", "-i", "pipe:0", "-c:a", "aac", "-b:a", "160k", dest],
                              stdin=subprocess.PIPE)
        with z.open(name) as f:
            while chunk := f.read(1 << 20):
                ff.stdin.write(chunk)
        ff.stdin.close()
        if ff.wait() != 0:
            sys.exit(f"ffmpeg failed on {name}")
        print(dest, "<-", name.split(" - ")[-1])
PY
fi
ls -l www/hold_web.wasm

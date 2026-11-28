#!/usr/bin/env python3
"""Serve a demo on localhost and save Henry's in-game feedback to a file.

A static file server like `python -m http.server`, plus one endpoint: a POST to
/feedback with a JSON object appends it, with a timestamp, as one line of the
feedback file (JSON Lines). The demo's feedback bar (keep / change / drop / note)
posts there, so Claude can read the verdict and the notes after Henry plays.

Usage:
  demo_server.py --dir <demo>/web/www --port 8765 --feedback <demo>/feedback.jsonl
"""

import argparse
import datetime
import functools
import http.server
import json
import os

MAX_BODY = 64 * 1024
KINDS = {"keep", "change", "drop", "note"}


class Handler(http.server.SimpleHTTPRequestHandler):
    feedback_path: str = ""  # set in main() before serving

    extensions_map = {
        **http.server.SimpleHTTPRequestHandler.extensions_map,
        ".wasm": "application/wasm",
        ".js": "text/javascript",
        ".m4a": "audio/mp4",
    }

    def end_headers(self):
        # A rebuilt demo should never be served stale.
        self.send_header("Cache-Control", "no-store")
        super().end_headers()

    def do_POST(self):
        if self.path != "/feedback":
            self.send_error(404)
            return
        length = int(self.headers.get("Content-Length") or 0)
        if length <= 0 or length > MAX_BODY:
            self.send_error(413 if length > MAX_BODY else 400)
            return
        try:
            entry = json.loads(self.rfile.read(length))
        except ValueError:
            self.send_error(400, "not JSON")
            return
        if not isinstance(entry, dict) or entry.get("kind") not in KINDS:
            self.send_error(400, "kind must be keep, change, drop or note")
            return
        entry = {"received": datetime.datetime.now().isoformat(timespec="seconds"), **entry}
        with open(self.feedback_path, "a", encoding="utf-8") as f:
            f.write(json.dumps(entry, ensure_ascii=False) + "\n")
        self.send_response(204)
        self.end_headers()


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--dir", required=True, help="the folder to serve (the demo's web/www)")
    ap.add_argument("--port", type=int, default=8765)
    ap.add_argument("--feedback", required=True, help="the JSON Lines file to append feedback to")
    args = ap.parse_args()

    Handler.feedback_path = os.path.abspath(args.feedback)
    handler = functools.partial(Handler, directory=os.path.abspath(args.dir))
    server = http.server.ThreadingHTTPServer(("127.0.0.1", args.port), handler)
    print(f"Serving {args.dir} at http://localhost:{args.port}/ ; feedback goes to {Handler.feedback_path}")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()

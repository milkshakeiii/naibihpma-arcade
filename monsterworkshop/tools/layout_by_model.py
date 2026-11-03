"""Layout by image model (DESIGN.md): ask an image model to recompose a place.

The loop: a functional draft of a place in the map, validated by the tests; its picture from
`monster_workshop --export-places`; this script, which sends that picture and a brief to an
image model and saves its variants; then the draft rebuilt to match the variant Henry picks,
with real tiles, and checked again.

    python tools/layout_by_model.py BRIEF [--variants N] [--model ID] [--plain]

BRIEF names a file in tools/briefs/ (without .txt). Its first line is the place's picture in
places/ (without .png); the rest is the prompt. Variants go to places/variants/. The key comes
from the GEMINI_API_KEY environment variable and is never printed. Only the standard library.
"""

import base64
import json
import os
import sys
import urllib.error
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
API = 'https://generativelanguage.googleapis.com/v1beta'
MODEL = 'gemini-nano-banana-2.1'
# A screen is 25 by 16 tiles: about 3:2.
ASPECT = '3:2'


def post(url, body, key, revision=None):
    headers = {'Content-Type': 'application/json', 'x-goog-api-key': key}
    if revision:
        headers['Api-Revision'] = revision
    req = urllib.request.Request(url, data=json.dumps(body).encode(), headers=headers, method='POST')
    with urllib.request.urlopen(req, timeout=300) as r:
        return json.load(r)


def images_and_text(node, images, texts):
    """Every image and text block anywhere in a response, in either API's shape."""
    if isinstance(node, dict):
        inline = node.get('inlineData') or node.get('inline_data')
        if isinstance(inline, dict) and isinstance(inline.get('data'), str):
            images.append((inline.get('mimeType') or inline.get('mime_type') or 'image/png', inline['data']))
        elif node.get('type') == 'image' and isinstance(node.get('data'), str):
            images.append((node.get('mime_type') or 'image/png', node['data']))
        elif isinstance(node.get('text'), str) and node.get('thought') is not True:
            texts.append(node['text'])
        for k, v in node.items():
            if k not in ('inlineData', 'inline_data') and not (k == 'data' and node.get('type') == 'image'):
                images_and_text(v, images, texts)
    elif isinstance(node, list):
        for v in node:
            images_and_text(v, images, texts)


def ask(model, prompt, png, key):
    """One variant: the Interactions API, or generateContent if that model only speaks it."""
    data = base64.b64encode(png).decode()
    try:
        return post(f'{API}/interactions', {
            'model': model,
            'input': [{'type': 'text', 'text': prompt}, {'type': 'image', 'mime_type': 'image/png', 'data': data}],
            'response_format': {'type': 'image', 'aspect_ratio': ASPECT, 'image_size': '2K'},
        }, key, revision='2026-05-20')
    except urllib.error.HTTPError as e:
        # The key, the account's credit or its rate: the other API won't do better.
        if e.code in (401, 402, 403, 429):
            raise
        print(f'  interactions said {e.code}: {e.read().decode(errors="replace")[:400]}')
        print('  trying generateContent')
    return post(f'{API}/models/{model}:generateContent', {
        'contents': [{'parts': [{'text': prompt}, {'inline_data': {'mime_type': 'image/png', 'data': data}}]}],
        'generationConfig': {'responseModalities': ['TEXT', 'IMAGE'], 'imageConfig': {'aspectRatio': ASPECT}},
    }, key)


def main():
    args = sys.argv[1:]
    if not args or args[0].startswith('-'):
        sys.exit(__doc__)
    opt = lambda flag, default: args[args.index(flag) + 1] if flag in args else default
    brief = args[0]
    variants, model = int(opt('--variants', '4')), opt('--model', MODEL)
    key = os.environ.get('GEMINI_API_KEY')
    if not key:
        sys.exit('Set GEMINI_API_KEY first (setx GEMINI_API_KEY ..., then open a new terminal).')
    lines = (HERE / 'briefs' / f'{brief}.txt').read_text(encoding='utf-8').splitlines()
    place, prompt = lines[0].strip(), '\n'.join(lines[1:]).strip()
    picture = ROOT / 'places' / (place + ('' if '--plain' in args else '_grid') + '.png')
    png = picture.read_bytes()
    out = ROOT / 'places' / 'variants'
    out.mkdir(parents=True, exist_ok=True)
    print(f'{brief}: {picture.name} to {model}, {variants} variants')
    for n in range(1, variants + 1):
        try:
            reply = ask(model, prompt, png, key)
        except urllib.error.HTTPError as e:
            sys.exit(f'  {e.code}: {e.read().decode(errors="replace")[:800]}')
        images, texts = [], []
        images_and_text(reply, images, texts)
        if not images:
            print(f'  variant {n}: no picture came back. {" ".join(texts)[:400]}')
            (out / f'{brief}_{n}.json').write_text(json.dumps(reply, indent=2)[:20000], encoding='utf-8')
            continue
        mime, b64 = images[-1]
        path = out / f'{brief}_{n}.{"jpg" if "jpeg" in mime else "png"}'
        path.write_bytes(base64.b64decode(b64))
        note = ' '.join(t.strip() for t in texts if t.strip())
        print(f'  variant {n}: {path.relative_to(ROOT)}' + (f'  ({note[:300]})' if note else ''))


if __name__ == '__main__':
    main()

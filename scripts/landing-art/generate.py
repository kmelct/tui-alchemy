#!/usr/bin/env python3
"""Generate a single landing-page art asset with OpenAI gpt-image-2.

Reads OPENAI_KEY from the repo .env. Saves the raw PNG to --out.
Usage:
  python3 scripts/landing-art/generate.py \
      --out website/assets/gen/raw/wax-seal.png \
      --size 1024x1024 --quality high --background transparent \
      --prompt "..."
"""
import argparse
import base64
import json
import os
import sys
import urllib.request
import urllib.error

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))


def read_key() -> str:
    env = os.path.join(ROOT, ".env")
    with open(env) as fh:
        for line in fh:
            if line.startswith("OPENAI_KEY="):
                return line.split("=", 1)[1].strip().strip('"').strip("'")
    raise SystemExit("OPENAI_KEY not found in .env")


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--prompt", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--size", default="1024x1024")  # 1024x1024 | 1536x1024 | 1024x1536
    ap.add_argument("--quality", default="high")     # high | medium | low
    ap.add_argument("--background", default="opaque")  # opaque | transparent
    args = ap.parse_args()

    body = {
        "model": "gpt-image-2",
        "prompt": args.prompt,
        "size": args.size,
        "quality": args.quality,
        "background": args.background,
        "output_format": "png",
        "n": 1,
    }
    req = urllib.request.Request(
        "https://api.openai.com/v1/images/generations",
        data=json.dumps(body).encode(),
        headers={
            "Authorization": f"Bearer {read_key()}",
            "Content-Type": "application/json",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=300) as resp:
            payload = json.load(resp)
    except urllib.error.HTTPError as exc:
        sys.stderr.write(f"HTTP {exc.code}: {exc.read().decode()[:1000]}\n")
        raise SystemExit(1)
    except urllib.error.URLError as exc:
        sys.stderr.write(f"request failed: {exc.reason}\n")
        raise SystemExit(1)

    try:
        b64 = payload["data"][0]["b64_json"]
    except (KeyError, IndexError, TypeError):
        sys.stderr.write(f"unexpected response shape: {json.dumps(payload)[:1000]}\n")
        raise SystemExit(1)

    out_path = os.path.join(ROOT, args.out)
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    with open(out_path, "wb") as fh:
        fh.write(base64.b64decode(b64))
    usage = payload.get("usage", {})
    print(f"OK {args.out}  ({args.size}, {args.quality}, bg={args.background})  usage={usage}")


if __name__ == "__main__":
    main()

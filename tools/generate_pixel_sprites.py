#!/usr/bin/env python3
"""Generate the local 32x32 pixel sprite atlas for tui-alchemy."""

from __future__ import annotations

import argparse
import base64
import json
import os
import sys
import tempfile
import time
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path


SCRIPT_ROOT = Path(__file__).resolve().parents[1]
OUTPUT_ROOT = Path.cwd() / "assets" / "pixel-sprites"
MANIFEST_PATH = OUTPUT_ROOT / "manifest.json"
DEFAULT_MODEL = "gpt-image-2"
PROMPT_VERSION = "pixel-atlas-v1"


@dataclass(frozen=True)
class SpriteJob:
    catalog: str
    slug: str
    name: str
    output_path: Path
    recipes: tuple[tuple[str, str], ...] = ()
    source_icon: Path | None = None
    frame: str = "base"


UI_JOBS = [
    ("catalog", "Little Alchemy recipe book"),
    ("combine", "combine selected elements"),
    ("clear", "clear current selection"),
    ("reset", "reset discovered elements"),
    ("hint", "show a discovery hint"),
]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--catalog", choices=["catalog", "ui", "all"], default="all")
    parser.add_argument("--limit", type=int, default=None)
    parser.add_argument("--only", action="append", default=[], help="Generate only this slug; repeatable.")
    parser.add_argument("--resume", action="store_true", help="Skip successful manifest entries with output files.")
    parser.add_argument("--dry-run", action="store_true", help="Plan generation without calling the API.")
    parser.add_argument("--force", action="store_true", help="Regenerate existing sprite files.")
    parser.add_argument("--model", default=DEFAULT_MODEL)
    args = parser.parse_args()

    api_key = load_api_key()
    if not api_key and not args.dry_run:
        print("OPENAI_API_KEY or OPENAI_KEY is required for generation.", file=sys.stderr)
        return 2

    manifest = load_manifest(args.model)
    jobs = select_jobs(args.catalog, set(args.only))
    if args.limit is not None:
        jobs = jobs[: max(args.limit, 0)]

    OUTPUT_ROOT.mkdir(parents=True, exist_ok=True)
    completed = completed_outputs(manifest) if args.resume else set()

    for job in expand_frame_jobs(jobs):
        status = plan_status(job, completed, args.force)
        if status == "skip existing":
            print(f"skip existing {display_path(job.output_path)}")
            record_manifest(manifest, args.model, job, "skipped", "existing output")
            continue
        if status == "skip manifest":
            print(f"skip manifest {display_path(job.output_path)}")
            record_manifest(manifest, args.model, job, "skipped", "resume completed")
            continue

        prompt = build_prompt(job)
        if args.dry_run:
            print(f"dry-run would generate {display_path(job.output_path)}")
            record_manifest(manifest, args.model, job, "dry-run", prompt=prompt)
            continue

        print(f"generating {display_path(job.output_path)}")
        try:
            image_bytes = generate_image(api_key or "", args.model, prompt)
            write_png_atomically(job.output_path, image_bytes)
            record_manifest(manifest, args.model, job, "succeeded", prompt=prompt)
        except Exception as error:  # noqa: BLE001 - preserve resumable failure in manifest.
            record_manifest(manifest, args.model, job, "failed", str(error), prompt)
            save_manifest(manifest)
            print(f"failed {job.slug}: {error}", file=sys.stderr)
            return 1

        save_manifest(manifest)
        time.sleep(0.2)

    save_manifest(manifest)
    return 0


def load_api_key() -> str | None:
    env = dict(os.environ)
    for env_file in [Path.cwd() / ".env", SCRIPT_ROOT / ".env"]:
        if env_file.exists():
            env.update(parse_env_file(env_file))
    return env.get("OPENAI_API_KEY") or env.get("OPENAI_KEY")


def parse_env_file(path: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    for line in path.read_text().splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#") or "=" not in stripped:
            continue
        key, value = stripped.split("=", 1)
        value = value.strip().strip('"').strip("'")
        values[key.strip()] = value
    return values


def select_jobs(catalog: str, only: set[str]) -> list[SpriteJob]:
    jobs: list[SpriteJob] = []
    if catalog in {"catalog", "all"}:
        jobs.extend(catalog_jobs())
    if catalog in {"ui", "all"}:
        jobs.extend(ui_jobs())
    if only:
        jobs = [job for job in jobs if job.slug in only]
    return jobs


def catalog_jobs() -> list[SpriteJob]:
    data = json.loads((SCRIPT_ROOT / "data" / "little_alchemy.json").read_text())
    jobs: list[SpriteJob] = []
    for element in data["elements"]:
        sprite_path = Path(element["sprite"])
        slug = slugify(element.get("slug") or element["name"])
        recipes = tuple(tuple(recipe) for recipe in element.get("recipes", []))
        source_icon = source_icon_for_sprite(sprite_path, slug)
        jobs.append(
            SpriteJob(
                catalog="catalog",
                slug=slug,
                name=element["name"],
                recipes=recipes,
                output_path=Path.cwd() / sprite_path,
                source_icon=source_icon,
            )
        )
    return jobs


def source_icon_for_sprite(sprite_path: Path, slug: str) -> Path | None:
    folder = sprite_path.parent.name
    extension = "svg" if folder.endswith("-2") else "png"
    candidate = SCRIPT_ROOT / "assets" / "icons" / folder / f"{slug}.{extension}"
    return candidate if candidate.exists() else None


def ui_jobs() -> list[SpriteJob]:
    return [
        SpriteJob("ui", slug, label, OUTPUT_ROOT / "ui" / f"{slug}.png")
        for slug, label in UI_JOBS
    ]


def expand_frame_jobs(jobs: list[SpriteJob]) -> list[SpriteJob]:
    expanded: list[SpriteJob] = []
    for job in jobs:
        expanded.append(job)
        if job.catalog == "ui":
            continue
        for frame in range(1, 4):
            expanded.append(
                SpriteJob(
                    catalog=job.catalog,
                    slug=f"{job.slug}_idle_{frame}",
                    name=f"{job.name} idle frame {frame}",
                    recipes=job.recipes,
                    output_path=job.output_path.with_name(
                        f"{job.output_path.stem}_idle_{frame}{job.output_path.suffix}"
                    ),
                    source_icon=job.source_icon,
                    frame=f"idle_{frame}",
                )
            )
    return expanded


def plan_status(job: SpriteJob, completed: set[str], force: bool) -> str:
    relative = display_path(job.output_path)
    if not force and job.output_path.exists():
        return "skip existing"
    if not force and relative in completed:
        return "skip manifest"
    return "generate"


def completed_outputs(manifest: dict) -> set[str]:
    return {
        entry["output_path"]
        for entry in manifest.get("entries", [])
        if entry.get("status") == "succeeded" and (Path.cwd() / entry["output_path"]).exists()
    }


def build_prompt(job: SpriteJob) -> str:
    recipe_text = ""
    if job.recipes:
        examples = [" + ".join(pair) for pair in job.recipes[:4]]
        recipe_text = f" Recipe clues: {', '.join(examples)}."
    source_text = ""
    if job.source_icon and job.source_icon.exists():
        source_text = f" A local source icon exists at {job.source_icon.relative_to(SCRIPT_ROOT)} for semantic reference."

    frame_text = (
        " This is the base readable idle pose."
        if job.frame == "base"
        else f" This is {job.frame}: keep the same silhouette and palette but shift flames, ripples, smoke, leaves, or sparkle pixels subtly so it loops with the base frame."
    )

    return (
        "Create a transparent 32x32 PNG pixel-art sprite for a terminal game atlas. "
        "Use the Lost Century 16-color palette: ink black, parchment cream, warm gold, "
        "rust red, leaf green, teal, denim blue, violet, bright cyan, white, and quiet grays. "
        "Use crisp 1-pixel edges, no anti-aliasing, no text, no border, no shadow, and a transparent background. "
        "The silhouette must stay readable when sampled to 8x8 terminal half-blocks. "
        "Make every sprite material-specific: water should use droplets/ripples, fire should use flame tongues and embers, "
        "air should use curling wind bands, earth should use soil mass and sprouts, and anything in a bottle/jar/glass should visibly sit inside a glass container. "
        f"Subject: {job.name}. Catalog: {job.catalog}.{frame_text}{recipe_text}{source_text}"
    )


def generate_image(api_key: str, model: str, prompt: str) -> bytes:
    body = json.dumps(
        {
            "model": model,
            "prompt": prompt,
            "n": 1,
            "size": "auto",
            "background": "transparent",
            "output_format": "png",
        }
    ).encode()
    request = urllib.request.Request(
        "https://api.openai.com/v1/images/generations",
        data=body,
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=180) as response:
            payload = json.loads(response.read().decode())
    except urllib.error.HTTPError as error:
        detail = error.read().decode(errors="replace")
        raise RuntimeError(f"OpenAI image request failed: HTTP {error.code} {detail}") from error

    image = payload.get("data", [{}])[0]
    if image.get("b64_json"):
        return base64.b64decode(image["b64_json"])
    if image.get("url"):
        with urllib.request.urlopen(image["url"], timeout=180) as response:
            return response.read()
    raise RuntimeError("OpenAI image response did not include b64_json or url")


def write_png_atomically(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(dir=path.parent, delete=False) as temp:
        temp.write(data)
        temp_path = Path(temp.name)
    temp_path.replace(path)


def load_manifest(model: str) -> dict:
    if MANIFEST_PATH.exists():
        return json.loads(MANIFEST_PATH.read_text())
    return {"prompt_version": PROMPT_VERSION, "model": model, "entries": []}


def record_manifest(
    manifest: dict,
    model: str,
    job: SpriteJob,
    status: str,
    message: str = "",
    prompt: str = "",
) -> None:
    manifest["prompt_version"] = PROMPT_VERSION
    manifest["model"] = model
    entries = manifest.setdefault("entries", [])
    entry = {
        "catalog": job.catalog,
        "slug": job.slug,
        "name": job.name,
        "frame": job.frame,
        "model": model,
        "prompt_version": PROMPT_VERSION,
        "output_path": display_path(job.output_path),
        "source_icon": display_path(job.source_icon) if job.source_icon else None,
        "status": status,
        "message": message,
        "updated_at": int(time.time()),
    }
    if prompt:
        entry["prompt"] = prompt

    for index, existing in enumerate(entries):
        if existing.get("catalog") == job.catalog and existing.get("slug") == job.slug:
            entries[index] = entry
            break
    else:
        entries.append(entry)


def save_manifest(manifest: dict) -> None:
    MANIFEST_PATH.parent.mkdir(parents=True, exist_ok=True)
    MANIFEST_PATH.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")


def display_path(path: Path | None) -> str:
    if path is None:
        return ""
    try:
        return str(path.relative_to(Path.cwd()))
    except ValueError:
        try:
            return str(path.relative_to(SCRIPT_ROOT))
        except ValueError:
            return str(path)


def slugify(value: str) -> str:
    out: list[str] = []
    pending_dash = False
    for char in value:
        if char.isascii() and char.isalnum():
            if pending_dash and out:
                out.append("-")
            out.append(char.lower())
            pending_dash = False
        else:
            pending_dash = True
    slug = "".join(out).strip("-")
    return slug or "element"


if __name__ == "__main__":
    raise SystemExit(main())

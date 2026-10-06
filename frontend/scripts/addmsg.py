#!/usr/bin/env python3
"""Merge messages into messages/en.json and messages/id.json.

  python3 scripts/addmsg.py < batch.json   # batch: {"key": ["English", "Indonesian"], ...}

A key that already exists with different text is an error, so two screens can't silently share a
key that means different things.
"""
import json, sys, pathlib

root = pathlib.Path(__file__).resolve().parent.parent / "messages"
batch = json.load(sys.stdin)
for index, locale in enumerate(["en", "id"]):
    path = root / f"{locale}.json"
    data = json.loads(path.read_text())
    for key, texts in batch.items():
        text = texts[index]
        if key in data and data[key] != text:
            sys.exit(f"{key} already exists in {locale}.json with different text: {data[key]!r} vs {text!r}")
        data[key] = text
    schema = data.pop("$schema", "https://inlang.com/schema/inlang-message-format")
    ordered = {"$schema": schema, **dict(sorted(data.items()))}
    path.write_text(json.dumps(ordered, ensure_ascii=False, indent="\t") + "\n")
print(f"{len(batch)} messages merged")

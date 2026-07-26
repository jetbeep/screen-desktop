#!/usr/bin/env python3
"""Encode an application profiles JSON file as JKV."""

from __future__ import annotations

import argparse
import json
import struct
import sys
from pathlib import Path

JKV_TAG = b"JKV"
JKV_VERSION = 1
TYPE_NULL = 1
TYPE_BOOL = 2
TYPE_POSITIVE_INT32 = 3
TYPE_NEGATIVE_INT32 = 4
TYPE_FLOAT = 5
TYPE_STRING = 6
TYPE_COLLECTION = 7
TYPE_ARRAY = 8
I32_MAX = 2**31 - 1
I32_MIN_SUPPORTED = -(2**31 - 1)


class JkvEncodeError(ValueError):
    pass


def encode_uint(value: int) -> bytes:
    body = value.to_bytes(4, "little")
    return body.rstrip(b"\0")


def encode_number(tag: int, value: int, output: bytearray) -> None:
    body = encode_uint(value)
    output.append((tag << 4) | len(body))
    output.extend(body)


def encode_string(value: str, output: bytearray) -> None:
    body = value.encode("utf-8")
    if b"\0" in body:
        raise JkvEncodeError("string contains an interior NUL byte")
    output.append(TYPE_STRING << 4)
    output.extend(body)
    output.append(0)


def encode_value(value: object, output: bytearray) -> None:
    if value is None:
        output.append(TYPE_NULL << 4)
    elif isinstance(value, bool):
        output.append((TYPE_BOOL << 4) | int(value))
    elif isinstance(value, int):
        if not I32_MIN_SUPPORTED <= value <= I32_MAX:
            raise JkvEncodeError(f"integer {value} is outside the supported i32 range")
        encode_number(TYPE_POSITIVE_INT32 if value >= 0 else TYPE_NEGATIVE_INT32, abs(value), output)
    elif isinstance(value, float):
        bits = struct.unpack("<I", struct.pack("<f", value))[0]
        encode_number(TYPE_FLOAT, bits, output)
    elif isinstance(value, str):
        encode_string(value, output)
    elif isinstance(value, list):
        encode_number(TYPE_ARRAY, len(value), output)
        for item in value:
            encode_value(item, output)
    elif isinstance(value, dict):
        encode_number(TYPE_COLLECTION, len(value), output)
        for key, item in value.items():
            if not isinstance(key, str):
                raise JkvEncodeError("object keys must be strings")
            encode_string(key, output)
            encode_value(item, output)
    else:
        raise JkvEncodeError(f"unsupported value type: {type(value)!r}")


def build(source: Path, destination: Path) -> int:
    data = json.loads(source.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise JkvEncodeError("profiles source must be a JSON object")
    if "active_network_id" not in data or "profiles" not in data:
        raise JkvEncodeError("profiles source requires active_network_id and profiles")
    if not isinstance(data["profiles"], list) or not data["profiles"]:
        raise JkvEncodeError("profiles must be a non-empty array")

    output = bytearray(JKV_TAG)
    output.append(JKV_VERSION)
    encode_value(data, output)
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(output)
    return len(output)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--src", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    try:
        size = build(args.src, args.out)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"build_profiles: error: {error}", file=sys.stderr)
        return 1
    print(f"build_profiles: wrote {size} bytes -> {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

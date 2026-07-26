#!/usr/bin/env python3
"""Convert a JSON file to a header-framed JKV binary file."""

from __future__ import annotations

import argparse
import json
import math
import struct
import sys
from pathlib import Path

JKV_HEADER = b"JKV\x01"
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
    return value.to_bytes(4, "little").rstrip(b"\0")


def encode_number(value_type: int, value: int, output: bytearray) -> None:
    body = encode_uint(value)
    output.append((value_type << 4) | len(body))
    output.extend(body)


def encode_string(value: str, output: bytearray) -> None:
    body = value.encode("utf-8")
    if b"\0" in body:
        raise JkvEncodeError("strings cannot contain NUL characters")
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
            raise JkvEncodeError(f"integer {value} is outside the supported JKV i32 range")
        value_type = TYPE_POSITIVE_INT32 if value >= 0 else TYPE_NEGATIVE_INT32
        encode_number(value_type, abs(value), output)
    elif isinstance(value, float):
        if not math.isfinite(value):
            raise JkvEncodeError("non-finite JSON numbers are not supported")
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
            encode_string(key, output)
            encode_value(item, output)
    else:
        raise JkvEncodeError(f"unsupported JSON value: {type(value).__name__}")


def convert(source: Path, destination: Path) -> int:
    data = json.loads(source.read_text(encoding="utf-8"), parse_constant=lambda value: (_ for _ in ()).throw(JkvEncodeError(f"invalid JSON number: {value}")))
    output = bytearray(JKV_HEADER)
    encode_value(data, output)
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(output)
    return len(output)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path, help="source JSON file")
    parser.add_argument("output", type=Path, help="destination JKV file")
    args = parser.parse_args()
    try:
        size = convert(args.input, args.output)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"json_to_jkv: error: {error}", file=sys.stderr)
        return 1
    print(f"json_to_jkv: wrote {size} bytes to {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
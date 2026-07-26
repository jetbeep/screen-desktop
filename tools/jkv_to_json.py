#!/usr/bin/env python3
"""Convert a header-framed JKV binary file to JSON."""

from __future__ import annotations

import argparse
import json
import struct
import sys
from pathlib import Path

JKV_HEADER = b"JKV\x01"
TYPE_UNDEFINED = 0
TYPE_NULL = 1
TYPE_BOOL = 2
TYPE_POSITIVE_INT32 = 3
TYPE_NEGATIVE_INT32 = 4
TYPE_FLOAT = 5
TYPE_STRING = 6
TYPE_COLLECTION = 7
TYPE_ARRAY = 8


class JkvDecodeError(ValueError):
    pass


class Decoder:
    def __init__(self, data: bytes) -> None:
        if not data.startswith(JKV_HEADER):
            raise JkvDecodeError("invalid or unsupported JKV header")
        self.data = data
        self.offset = len(JKV_HEADER)

    def read_byte(self) -> int:
        if self.offset >= len(self.data):
            raise JkvDecodeError("unexpected end of file")
        value = self.data[self.offset]
        self.offset += 1
        return value

    def read_uint(self, width: int) -> int:
        if width > 4:
            raise JkvDecodeError(f"invalid integer width: {width}")
        end = self.offset + width
        if end > len(self.data):
            raise JkvDecodeError("unexpected end of file")
        value = int.from_bytes(self.data[self.offset:end], "little")
        self.offset = end
        return value

    def read_string(self) -> str:
        end = self.data.find(b"\0", self.offset)
        if end < 0:
            raise JkvDecodeError("unterminated string")
        try:
            value = self.data[self.offset:end].decode("utf-8")
        except UnicodeDecodeError as error:
            raise JkvDecodeError(f"invalid UTF-8 string: {error}") from error
        self.offset = end + 1
        return value

    def read_value(self) -> object:
        descriptor = self.read_byte()
        value_type = descriptor >> 4
        low = descriptor & 0x0F

        if value_type == TYPE_UNDEFINED:
            raise JkvDecodeError("JKV undefined has no JSON representation")
        if value_type == TYPE_NULL:
            return None
        if value_type == TYPE_BOOL:
            return bool(low & 1)
        if value_type == TYPE_POSITIVE_INT32:
            value = self.read_uint(low)
            if value > 2**31 - 1:
                raise JkvDecodeError("positive integer is outside the i32 range")
            return value
        if value_type == TYPE_NEGATIVE_INT32:
            value = self.read_uint(low)
            if value > 2**31 - 1:
                raise JkvDecodeError("negative integer magnitude is outside the i32 range")
            return -value
        if value_type == TYPE_FLOAT:
            bits = self.read_uint(low)
            return struct.unpack("<f", struct.pack("<I", bits))[0]
        if value_type == TYPE_STRING:
            return self.read_string()
        if value_type == TYPE_ARRAY:
            return [self.read_value() for _ in range(self.read_uint(low))]
        if value_type == TYPE_COLLECTION:
            result = {}
            for _ in range(self.read_uint(low)):
                key_descriptor = self.read_byte()
                key_type = key_descriptor >> 4
                key_low = key_descriptor & 0x0F
                if key_type == TYPE_STRING:
                    key = self.read_string()
                elif key_type == TYPE_POSITIVE_INT32:
                    key = str(self.read_uint(key_low))
                elif key_type == TYPE_NEGATIVE_INT32:
                    key = str(-self.read_uint(key_low))
                else:
                    raise JkvDecodeError("collection key is not a string or integer")
                result[key] = self.read_value()
            return result
        raise JkvDecodeError(f"invalid JKV type: {value_type}")


def convert(source: Path, destination: Path) -> None:
    decoder = Decoder(source.read_bytes())
    value = decoder.read_value()
    if decoder.offset != len(decoder.data):
        raise JkvDecodeError("trailing bytes after top-level value")
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path, help="source JKV file")
    parser.add_argument("output", type=Path, help="destination JSON file")
    args = parser.parse_args()
    try:
        convert(args.input, args.output)
    except (OSError, ValueError) as error:
        print(f"jkv_to_json: error: {error}", file=sys.stderr)
        return 1
    print(f"jkv_to_json: wrote {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
#!/usr/bin/env python3
"""
Convert cel-spec textproto test data files to binary protobuf (.binpb).

The rust-protobuf text format parser doesn't support all textproto features
used in the cel-spec test data (expanded Any, colon-before-brace, etc.).
Google's Python protobuf library handles all of these correctly, so we use
it to parse and re-serialize as binary proto which Rust can read without issues.

Usage:
    .venv/bin/python3 convert_testdata.py
"""

import sys
import os

sys.path.insert(0, os.path.dirname(__file__))

import cel_spec_tests.proto.simple_pb2 as simple
import cel_spec_tests.proto.test_all_types_proto2_pb2
import cel_spec_tests.proto.test_all_types_proto3_pb2
from google.protobuf.text_format import Parse

TEST_DATA_DIR = os.path.join(os.path.dirname(__file__), "cel_spec_tests", "simple-test-data")

converted = 0
failed = 0

for filename in sorted(os.listdir(TEST_DATA_DIR)):
    if not filename.endswith(".textproto"):
        continue

    src = os.path.join(TEST_DATA_DIR, filename)
    dst = os.path.join(TEST_DATA_DIR, filename.replace(".textproto", ".binpb"))

    with open(src) as f:
        data = f.read()

    try:
        msg = Parse(data, simple.SimpleTestFile(),
                    allow_unknown_field=True,
                    allow_unknown_extension=True)
        with open(dst, "wb") as f:
            f.write(msg.SerializeToString())
        converted += 1
        print(f"OK  {filename}")
    except Exception as e:
        failed += 1
        print(f"ERR {filename}: {e}")
        print(f"    NOTE: tests in this file will not run until conversion succeeds.")

print(f"\n{converted} converted, {failed} failed")

# -*- coding: utf-8 -*-
# Generated by the protocol buffer compiler.  DO NOT EDIT!
# source: simple.proto
# Protobuf Python Version: 4.25.3
"""Generated protocol buffer code."""
from google.protobuf import descriptor as _descriptor
from google.protobuf import descriptor_pool as _descriptor_pool
from google.protobuf import symbol_database as _symbol_database
from google.protobuf.internal import builder as _builder
# @@protoc_insertion_point(imports)

_sym_db = _symbol_database.Default()


from google.api.expr.v1alpha1 import checked_pb2 as google_dot_api_dot_expr_dot_v1alpha1_dot_checked__pb2
from google.api.expr.v1alpha1 import eval_pb2 as google_dot_api_dot_expr_dot_v1alpha1_dot_eval__pb2
from google.api.expr.v1alpha1 import value_pb2 as google_dot_api_dot_expr_dot_v1alpha1_dot_value__pb2


DESCRIPTOR = _descriptor_pool.Default().AddSerializedFile(b'\n\x0csimple.proto\x12\x17google.api.expr.test.v1\x1a&google/api/expr/v1alpha1/checked.proto\x1a#google/api/expr/v1alpha1/eval.proto\x1a$google/api/expr/v1alpha1/value.proto\"p\n\x0eSimpleTestFile\x12\x0c\n\x04name\x18\x01 \x01(\t\x12\x13\n\x0b\x64\x65scription\x18\x02 \x01(\t\x12;\n\x07section\x18\x03 \x03(\x0b\x32*.google.api.expr.test.v1.SimpleTestSection\"i\n\x11SimpleTestSection\x12\x0c\n\x04name\x18\x01 \x01(\t\x12\x13\n\x0b\x64\x65scription\x18\x02 \x01(\t\x12\x31\n\x04test\x18\x03 \x03(\x0b\x32#.google.api.expr.test.v1.SimpleTest\"\x8c\x05\n\nSimpleTest\x12\x0c\n\x04name\x18\x01 \x01(\t\x12\x13\n\x0b\x64\x65scription\x18\x02 \x01(\t\x12\x0c\n\x04\x65xpr\x18\x03 \x01(\t\x12\x16\n\x0e\x64isable_macros\x18\x04 \x01(\x08\x12\x15\n\rdisable_check\x18\x05 \x01(\x08\x12\x30\n\x08type_env\x18\x06 \x03(\x0b\x32\x1e.google.api.expr.v1alpha1.Decl\x12\x11\n\tcontainer\x18\r \x01(\t\x12\x43\n\x08\x62indings\x18\x07 \x03(\x0b\x32\x31.google.api.expr.test.v1.SimpleTest.BindingsEntry\x12\x30\n\x05value\x18\x08 \x01(\x0b\x32\x1f.google.api.expr.v1alpha1.ValueH\x00\x12\x38\n\neval_error\x18\t \x01(\x0b\x32\".google.api.expr.v1alpha1.ErrorSetH\x00\x12\x43\n\x0f\x61ny_eval_errors\x18\n \x01(\x0b\x32(.google.api.expr.test.v1.ErrorSetMatcherH\x00\x12\x37\n\x07unknown\x18\x0b \x01(\x0b\x32$.google.api.expr.v1alpha1.UnknownSetH\x00\x12\x42\n\x0c\x61ny_unknowns\x18\x0c \x01(\x0b\x32*.google.api.expr.test.v1.UnknownSetMatcherH\x00\x1aT\n\rBindingsEntry\x12\x0b\n\x03key\x18\x01 \x01(\t\x12\x32\n\x05value\x18\x02 \x01(\x0b\x32#.google.api.expr.v1alpha1.ExprValue:\x02\x38\x01\x42\x10\n\x0eresult_matcher\"E\n\x0f\x45rrorSetMatcher\x12\x32\n\x06\x65rrors\x18\x01 \x03(\x0b\x32\".google.api.expr.v1alpha1.ErrorSet\"K\n\x11UnknownSetMatcher\x12\x36\n\x08unknowns\x18\x01 \x03(\x0b\x32$.google.api.expr.v1alpha1.UnknownSetB+Z)github.com/google/cel-spec/test/v1/testpbb\x06proto3')

_globals = globals()
_builder.BuildMessageAndEnumDescriptors(DESCRIPTOR, _globals)
_builder.BuildTopDescriptorsAndMessages(DESCRIPTOR, 'simple_pb2', _globals)
if _descriptor._USE_C_DESCRIPTORS == False:
  _globals['DESCRIPTOR']._options = None
  _globals['DESCRIPTOR']._serialized_options = b'Z)github.com/google/cel-spec/test/v1/testpb'
  _globals['_SIMPLETEST_BINDINGSENTRY']._options = None
  _globals['_SIMPLETEST_BINDINGSENTRY']._serialized_options = b'8\001'
  _globals['_SIMPLETESTFILE']._serialized_start=156
  _globals['_SIMPLETESTFILE']._serialized_end=268
  _globals['_SIMPLETESTSECTION']._serialized_start=270
  _globals['_SIMPLETESTSECTION']._serialized_end=375
  _globals['_SIMPLETEST']._serialized_start=378
  _globals['_SIMPLETEST']._serialized_end=1030
  _globals['_SIMPLETEST_BINDINGSENTRY']._serialized_start=928
  _globals['_SIMPLETEST_BINDINGSENTRY']._serialized_end=1012
  _globals['_ERRORSETMATCHER']._serialized_start=1032
  _globals['_ERRORSETMATCHER']._serialized_end=1101
  _globals['_UNKNOWNSETMATCHER']._serialized_start=1103
  _globals['_UNKNOWNSETMATCHER']._serialized_end=1178
# @@protoc_insertion_point(module_scope)

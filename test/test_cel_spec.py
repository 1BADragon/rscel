import os
import cel_spec_tests.proto.simple_pb2 as simple
from google.protobuf.text_format import Parse
import cel_spec_tests.proto.test_all_types_proto2_pb2
import cel_spec_tests.proto.test_all_types_proto3_pb2
import rscel
import pytest

debug_enabled = True

def debug(s, end='\n'):
    if debug_enabled:
        print(s, end=end)

def collect_tests():
    n_fails = 0
    test_data_path = os.path.join(os.path.dirname(__file__), 'cel_spec_tests/simple-test-data')
    for file in os.listdir(test_data_path):
        try:
            if file.endswith('textproto'):
                with open(os.path.join(test_data_path, file)) as f:
                    data = f.read()
                    sample_data = Parse(data, simple.SimpleTestFile())
            
                    for section in sample_data.section:
                        for test in section.test:
                            yield ((f'{file}::{section.name}::{test.name}', test))
        except:
            n_fails += 1
    
@pytest.mark.parametrize('name,test', list(collect_tests()))
def test_cel_spec(name, test):

    bindings = {}
    for key, value in test.bindings.items():
        if value.HasField('value'):
            bindings[key] = normalize_wrapped_value(value.value)
        else:
            bindings[key] = value


    if test.HasField('eval_error'):
        with pytest.raises(Exception):
            rscel.eval(test.expr, bindings)
    else:
        res = rscel.eval(test.expr, bindings)

        assert res == normalize_wrapped_value(test.value)


def normalize_wrapped_value(value):
        if value.HasField('int64_value'):
            return value.int64_value
        elif value.HasField('uint64_value'):
            return value.uint64_value
        elif value.HasField('double_value'):
            return value.double_value
        elif value.HasField('bytes_value'):
            return value.bytes_value
        elif value.HasField('bool_value'):
            return value.bool_value
        elif value.HasField('null_value'):
            return None
        elif value.HasField('string_value'):
            return value.string_value
        elif value.HasField('list_value'):
            return [normalize_wrapped_value(v) for v in value.list_value.values]
        elif value.HasField('map_value'):
            m = {}
            for entry in value.map_value.entries:
                m[normalize_wrapped_value(entry.key)] = normalize_wrapped_value(entry.value)

            return m
        else:
            raise Exception(f"Unknown value: {value}")
                


        


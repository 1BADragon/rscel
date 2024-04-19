import pytest
import datetime as dt
from zoneinfo import ZoneInfo
import rscel


class TestIntWrapper:
    def __init__(self, val: int):
        self._val = val

    def __eq__(self, other):
        if isinstance(other, TestIntWrapper):
            return self._val == other._val
        return False


class NestedClass:
    class Inner:
        def __init__(self):
            self.a = "foo"

    def __init__(self):
        self.foo = NestedClass.Inner()


def test_basic_equation():
    assert rscel.eval("3 + 3", {}) == 6


def test_obj_eq():
    assert rscel.eval("e == e", {"e": TestIntWrapper(3)})
    assert rscel.eval("e._val", {"e": TestIntWrapper(3)}) == 3


def test_nested_obj():
    assert rscel.eval("f.foo.a", {"f": NestedClass()}) == "foo"


def test_callable_arg():
    called = False

    def func():
        nonlocal called
        called = True
        return None

    assert rscel.eval("f()", {"f": func}) == None
    assert called


def test_datetime_converstion():
    d = dt.datetime.now(tz=ZoneInfo("America/Los_Angeles"))
    res = rscel.eval("d", {"d": d})
    assert res == d

    d = dt.datetime.now()
    res = rscel.eval("d", {"d": d})
    assert res == d.astimezone(dt.timezone.utc)

def test_empty_program():
    p = rscel.CelProgram()

    with pytest.raises(ValueError):
        rscel.CelContext().add_program('empty', p)

def test_program_bad_source():
    p = rscel.CelProgram()

    with pytest.raises(ValueError):
        p.add_source("3 +")

def test_program():
    c = rscel.CelContext()
    b = rscel.BindContext()
    p = rscel.CelProgram()

    p.add_source('3 + 3')
    c.add_program('entry', p)

    assert c.exec('entry', b) == 6

def test_program_json_serialize():
    c = rscel.CelContext()
    b = rscel.BindContext()
    p1 = rscel.CelProgram()

    p1.add_source('3 + 3')

    s = p1.serialize_to_json()

    p2 = rscel.CelProgram()
    p2.add_serialized_json(s)

    c.add_program('entry', p2)

    assert c.exec('entry', b) == 6
    
def test_program_bincode_serialize():
    c = rscel.CelContext()
    b = rscel.BindContext()
    p1 = rscel.CelProgram()

    p1.add_source('3 + 3')

    s = p1.serialize_to_bincode()

    p2 = rscel.CelProgram()
    p2.add_serialized_bincode(s)

    c.add_program('entry', p2)

    assert c.exec('entry', b) == 6

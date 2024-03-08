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

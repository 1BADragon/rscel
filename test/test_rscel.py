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


def test_obj_neq_null():
    # Regression: a bound Python object compared to null used to panic
    # (neq() hit unreachable!() because CelPyObject::eq returned an error for
    # `obj == null`). A Python object is never equal to null.
    assert rscel.eval("e == null", {"e": TestIntWrapper(3)}) == False
    assert rscel.eval("e != null", {"e": TestIntWrapper(3)}) == True
    # Null-guard chains used by filters must short-circuit cleanly.
    assert rscel.eval("e != null && e._val == 3", {"e": TestIntWrapper(3)}) == True


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
        rscel.CelContext().add_program("empty", p)


def test_program_bad_source():
    p = rscel.CelProgram()

    with pytest.raises(ValueError):
        p.add_source("3 +")


def test_program():
    c = rscel.CelContext()
    b = rscel.BindContext()
    p = rscel.CelProgram()

    p.add_source("3 + 3")
    c.add_program("entry", p)

    assert c.exec("entry", b) == 6


def test_program_json_serialize():
    c = rscel.CelContext()
    b = rscel.BindContext()
    p1 = rscel.CelProgram()

    p1.add_source("3 + 3")

    s = p1.serialize_to_json()

    p2 = rscel.CelProgram()
    p2.add_serialized_json(s)

    c.add_program("entry", p2)

    assert c.exec("entry", b) == 6


FIXED_START = dt.datetime(2020, 5, 17, 12, 30, 0, tzinfo=dt.timezone.utc)


def test_runtime_start_time_roundtrip():
    rt = rscel.InterpRuntime(FIXED_START)
    assert rt.start_time == FIXED_START
    assert rt.start_time.tzinfo is not None


def test_runtime_converts_other_timezones_to_utc():
    eastern = FIXED_START.astimezone(ZoneInfo("America/New_York"))
    rt = rscel.InterpRuntime(eastern)
    assert rt.start_time == FIXED_START
    assert rt.start_time.utcoffset() == dt.timedelta(0)


def test_runtime_default_is_now():
    before = dt.datetime.now(dt.timezone.utc)
    rt = rscel.InterpRuntime()
    after = dt.datetime.now(dt.timezone.utc)
    assert (
        before - dt.timedelta(milliseconds=1)
        <= rt.start_time
        <= after + dt.timedelta(milliseconds=1)
    )


def test_runtime_rejects_non_datetime():
    with pytest.raises(ValueError):
        rscel.InterpRuntime("2020-05-17")


def test_runtime_ns_roundtrip_keeps_full_precision():
    ns = 1_589_718_600_123_456_789
    rt = rscel.InterpRuntime.from_ns(ns)
    assert rt.start_time_ns == ns
    assert rscel.InterpRuntime.from_ns(rt.start_time_ns) == rt


def test_runtime_equality():
    assert rscel.InterpRuntime(FIXED_START) == rscel.InterpRuntime(FIXED_START)
    assert rscel.InterpRuntime(FIXED_START) != rscel.InterpRuntime(
        FIXED_START + dt.timedelta(seconds=1)
    )


def test_eval_with_runtime_fixes_now():
    rt = rscel.InterpRuntime(FIXED_START)
    assert rscel.eval("now()", {}, runtime=rt) == FIXED_START
    # The same runtime replays the same time.
    assert rscel.eval("now()", {}, runtime=rt) == FIXED_START


def test_eval_without_runtime_uses_current_time():
    res = rscel.eval("now()", {})
    assert abs(dt.datetime.now(dt.timezone.utc) - res) < dt.timedelta(seconds=5)


def test_eval_clock_now_ignores_runtime():
    rt = rscel.InterpRuntime(FIXED_START)
    assert rscel.eval("clockNow()", {}, runtime=rt) != FIXED_START


def test_exec_with_runtime_fixes_now():
    c = rscel.CelContext()
    b = rscel.BindContext()
    c.add_program_str("entry", "now()")

    rt = rscel.InterpRuntime(FIXED_START)
    assert c.exec("entry", b, runtime=rt) == FIXED_START
    assert c.exec("entry", b, runtime=rt) == FIXED_START


def test_exec_runtime_is_keyword_only():
    c = rscel.CelContext()
    b = rscel.BindContext()
    c.add_program_str("entry", "now()")

    with pytest.raises(TypeError):
        c.exec("entry", b, rscel.InterpRuntime(FIXED_START))


def test_runtime_nested_callback_uses_its_own_runtime():
    inner_start = dt.datetime(1999, 12, 31, 23, 59, 59, tzinfo=dt.timezone.utc)

    def inner():
        return rscel.eval("now()", {}, runtime=rscel.InterpRuntime(inner_start))

    rt = rscel.InterpRuntime(FIXED_START)
    res = rscel.eval("[now(), inner(), now()]", {"inner": inner}, runtime=rt)
    assert res == [FIXED_START, inner_start, FIXED_START]

import {
  celDetails,
  celEval,
  CelContext,
  BindContext,
  CelProgram,
  celEvalWithRuntime,
  InterpRuntime,
} from "rscel_wasm";
import { describe, it, expect } from "vitest";

describe("Rscel Basic Tests", () => {
  it("should be able to run a basic test case", () => {
    const res = celEval("3+3", {});

    expect(res.result()).toEqual(6n);
  });

  it("date passthrough", () => {
    const now = new Date();

    const res = celEval("foo", { foo: now });
    expect(res.result()).toEqual(now);
  });

  it("should be able to add a duration", () => {
    const now = new Date();
    const next = new Date(now);
    next.setSeconds(next.getSeconds() + 5);

    const res = celEval("foo + duration('5s')", { foo: now });
    expect(res.result()).toEqual(next);
  });

  it("should return a duration as an object", () => {
    const res = celEval("duration('5s')", {});

    expect(res.result()).toEqual({ sec: 5n, nsec: 0 });
  });

  it("Should serialize nested objects correctly", () => {
    const res = celEval("foo.bar", { foo: { bar: 4 } });

    expect(res.result()).toEqual(4n);
  });

  it("Should return nested objects correctly", () => {
    const res = celEval("foo.bar", { foo: { bar: { baz: 3 } } });

    expect(res.result()).toEqual({ baz: 3n });
  });

  it("int type", () => {
    const res = celEval("type(foo)", { foo: 3n });

    expect(res.result().type).toEqual("int");
  });

  it("float type", () => {
    const res = celEval("type(foo)", { foo: 3.1 });

    expect(res.result().type).toEqual("float");
  });

  it("string type", () => {
    const res = celEval("type(foo)", { foo: "foo" });

    expect(res.result().type).toEqual("string");
  });

  it("date type", () => {
    const res = celEval("type(foo)", { foo: new Date() });

    expect(res.result().type).toEqual("timestamp");
  });

  it("bool type", () => {
    const res = celEval("type(foo)", { foo: true });

    expect(res.result().type).toEqual("bool");
  });

  it("null type", () => {
    const res = celEval("type(foo)", { foo: null });

    expect(res.result().type).toEqual("null");
  });

  it("int type pun", () => {
    const res = celEval("type(foo)", { foo: { cel_int: 4 } });

    expect(res.result().type).toEqual("int");
  });

  it("float type pun", () => {
    const res = celEval("type(foo)", { foo: { cel_float: 4 } });

    expect(res.result().type).toEqual("float");
  });

  it("uint type pun", () => {
    const res = celEval("type(foo)", { foo: { cel_uint: 5 } });

    expect(res.result().type).toEqual("uint");
  });

  it("details works", () => {
    const dets = celDetails("3 + foo");

    expect(dets.details().params).toContain("foo");
  });

  it("detials syntax error", () => {
    const dets = celDetails("3 +");

    expect(dets.isSuccess()).toBe(false);
  });

  it("context supports multiple programs", () => {
    const ctx = new CelContext();
    ctx.addProgramStr("one", "foo + 1");
    ctx.addProgramStr("two", "double(foo)");

    const params = new BindContext();
    params.bindParam("foo", 3n);

    const resOne = ctx.exec("one", params);
    expect(resOne.result()).toEqual(4n);

    const bindings = new BindContext();
    bindings.bindParam("foo", 4n);
    bindings.bindFunc("double", (value) => {
      const v = typeof value === "bigint" ? value : BigInt(value);
      return v * 2n;
    });

    const resTwo = ctx.exec("two", bindings);
    expect(resTwo.result()).toEqual(8n);
  });

  it("can execute program instances", () => {
    const program = new CelProgram();
    program.addSource("triple(x)");

    const ctx = new CelContext();
    ctx.addProgram("triple", program);

    const bindings = new BindContext();
    bindings.bindFunc("triple", (value) => {
      const v = typeof value === "bigint" ? value : BigInt(value);
      return v * 3n;
    });
    bindings.bindParam("x", 2n);

    const result = ctx.exec("triple", bindings);
    expect(result.result()).toEqual(6n);
  });
});

describe("InterpRuntime", () => {
  const FIXED_START = new Date(Date.UTC(2020, 4, 17, 12, 30, 0, 123));

  it("round trips its start time as a Date", () => {
    const rt = new InterpRuntime(FIXED_START);
    expect(rt.startTime).toEqual(FIXED_START);
  });

  it("defaults to the current time", () => {
    const before = Date.now();
    const rt = new InterpRuntime();
    const after = Date.now();
    expect(rt.startTime.getTime()).toBeGreaterThanOrEqual(before);
    expect(rt.startTime.getTime()).toBeLessThanOrEqual(after);
  });

  it("rejects an invalid Date", () => {
    expect(() => new InterpRuntime(new Date("not a date"))).toThrow();
  });

  it("round trips nanoseconds through ISO strings", () => {
    const iso = "2020-05-17T12:30:00.123456789Z";
    const rt = InterpRuntime.fromIso(iso);
    expect(rt.startTimeIso).toEqual(iso);
    expect(InterpRuntime.fromIso(rt.startTimeIso).equals(rt)).toBe(true);
  });

  it("normalizes ISO offsets to UTC", () => {
    const rt = InterpRuntime.fromIso("2020-05-17T08:30:00.123-04:00");
    expect(rt.startTime).toEqual(FIXED_START);
    expect(rt.startTimeIso).toEqual("2020-05-17T12:30:00.123000000Z");
  });

  it("rejects an invalid ISO string", () => {
    expect(() => InterpRuntime.fromIso("yesterday")).toThrow();
  });

  it("compares by start time", () => {
    const a = new InterpRuntime(FIXED_START);
    expect(a.equals(new InterpRuntime(FIXED_START))).toBe(true);
    expect(a.equals(new InterpRuntime(new Date(0)))).toBe(false);
  });

  it("celEvalWithRuntime fixes now() and can be reused", () => {
    const rt = new InterpRuntime(FIXED_START);
    expect(celEvalWithRuntime("now()", {}, rt).result()).toEqual(FIXED_START);
    // The runtime is not consumed, so it replays the same time.
    expect(celEvalWithRuntime("now()", {}, rt).result()).toEqual(FIXED_START);
    expect(rt.startTime).toEqual(FIXED_START);
  });

  it("celEval without a runtime uses the current time", () => {
    const res = celEval("now()", {}).result();
    expect(Math.abs(Date.now() - res.getTime())).toBeLessThan(5000);
  });

  it("clockNow() ignores the runtime", () => {
    const rt = new InterpRuntime(FIXED_START);
    const res = celEvalWithRuntime("clockNow()", {}, rt).result();
    expect(res).not.toEqual(FIXED_START);
  });

  it("CelContext.execWithRuntime fixes now() and can be reused", () => {
    const ctx = new CelContext();
    ctx.addProgramStr("entry", "now()");
    const bindings = new BindContext();
    const rt = new InterpRuntime(FIXED_START);

    expect(ctx.execWithRuntime("entry", bindings, rt).result()).toEqual(FIXED_START);
    expect(ctx.execWithRuntime("entry", bindings, rt).result()).toEqual(FIXED_START);
  });

  it("a nested callback's runtime wins and the outer one is restored", () => {
    const innerStart = new Date(Date.UTC(1999, 11, 31, 23, 59, 59));
    const inner = () =>
      celEvalWithRuntime("now()", {}, new InterpRuntime(innerStart)).result();

    const rt = new InterpRuntime(FIXED_START);
    const res = celEvalWithRuntime("[now(), inner(), now()]", { inner }, rt);
    expect(res.result()).toEqual([FIXED_START, innerStart, FIXED_START]);
  });
});

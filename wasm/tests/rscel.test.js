import { celDetails, celEval } from "rscel_wasm";
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
});

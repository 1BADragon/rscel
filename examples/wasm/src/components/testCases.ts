import { cel_eval } from "rscel";

export const tests: [string, () => any, any][] = [
  ["basic", () => cel_eval("3+3", {}), BigInt(6)],
  ["cel_int", () => cel_eval("3 + foo", { foo: { cel_int: 4.3 } }), BigInt(7)],
  ["cel_float", () => cel_eval("3.0 + foo", { foo: { cel_float: 2 } }), 5.0],
  [
    "cel_uint",
    () => cel_eval("3u + foo", { foo: { cel_uint: 2.1 } }),
    BigInt(5),
  ],
  ["float", () => cel_eval("3.2 + foo", { foo: 2.1 }), 5.3],
  ["int", () => cel_eval("3 + foo", { foo: 3 }), BigInt(6)],
];

import { celEval } from "rscel";

export const tests: [string, () => any, any][] = [
  ["basic", () => celEval("3+3", {}), BigInt(6)],
  ["cel_int", () => celEval("3 + foo", { foo: { cel_int: 4.3 } }), BigInt(7)],
  ["cel_float", () => celEval("3.0 + foo", { foo: { cel_float: 2 } }), 5.0],
  [
    "cel_uint",
    () => celEval("3u + foo", { foo: { cel_uint: 2.1 } }),
    BigInt(5),
  ],
  ["float", () => celEval("3.2 + foo", { foo: 2.1 }), 5.3],
  ["int", () => celEval("3 + foo", { foo: 3 }), BigInt(6)],
  ["bigint", () => celEval("3 + foo", { foo: BigInt(5) }), BigInt(8)],
];

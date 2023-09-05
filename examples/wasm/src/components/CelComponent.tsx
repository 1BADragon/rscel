import * as React from "react";
import "./CelComponent.css";

import init, { cel_eval, cel_details } from "rscel";
import { useState, useEffect } from "react";

export default function CelComponent() {
  const [errorMessage, setErrorMessage] = useState<string>("");
  const [prog, setProg] = useState<string>("");
  const [params, setParams] = useState<string[]>([]);
  const [paramVals, setParamVals] = useState<any>({});
  const [result, setResult] = useState<any | undefined>(undefined);

  const generateParams = (): JSX.Element[] => {
    return params.map((val) => {
      return (
        <div key={`param-${val}`} style={{ display: "flex" }}>
          <label>{val}</label>
          <input
            style={{ marginLeft: "auto" }}
            onChange={(event) => {
              setParamVals((old: any) => {
                try {
                  let newObj = { ...old };
                  newObj[val] = JSON.parse(event.target.value, (_, val) => {
                    console.log(val);
                    return typeof val === "string" && val.endsWith("n")
                      ? BigInt(val.slice(0, -1))
                      : val;
                  });
                  setErrorMessage("");
                  return newObj;
                } catch (e) {
                  setErrorMessage(e.toString());
                  return old;
                }
              });
            }}
          />
        </div>
      );
    });
  };

  return (
    <div style={{ margin: "15px" }}>
      <h4>RsCel Evaluater</h4>
      <label>Program Source:</label>
      <textarea
        style={{ width: "100%", height: "100px" }}
        onChange={(event) => {
          setProg(event.target.value);
        }}
      />
      <div style={{ display: "flex", rowGap: "10px", justifyContent: "right" }}>
        <button
          onClick={() => {
            const res = cel_details(prog);

            if (res.success) {
              console.log(res);
              const details = res.details;
              setParams(details.params);
              setErrorMessage("");
              setResult(res.result);
            } else {
              setErrorMessage(
                res.error
                  ? `${res.error.kind}: ${res.error.msg}`
                  : "Unknown error",
              );
              setResult(undefined);
            }
          }}
        >
          Analyze
        </button>
        <button
          onClick={() => {
            console.log(paramVals);
            const result = cel_eval(prog, paramVals);
            console.log(result);

            if (result.success) {
              setErrorMessage(
                `Result: ${JSON.stringify(result.result, (_, v) =>
                  typeof v === "bigint" ? v.toString() : v,
                )}`,
              );
            } else {
              setErrorMessage(
                result.error
                  ? `${result.error.kind}: ${result.error.msg}`
                  : "Unknown error",
              );
            }
          }}
        >
          Eval
        </button>
      </div>
      <label>{errorMessage}</label>
      <div style={{ marginTop: "40px" }}>{generateParams()}</div>
    </div>
  );
}

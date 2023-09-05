import * as React from "react";
import "./CelComponent.css";

import init, { cel_eval, cel_details } from "rscel";
import { useState, useEffect } from "react";

export default function CelComponent() {
  const [errorMessage, setErrorMessage] = useState<string>("");
  const [prog, setProg] = useState<string>("");
  const [params, setParams] = useState<string[]>([]);
  const [paramVals, setParamVals] = useState<any>({});
  const [lastResult, setLastResult] = useState<any | undefined>(undefined);

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

  const renderResult = (currResult: any): JSX.Element => {
    console.log("render", currResult, typeof currResult);
    switch (typeof currResult) {
      case "number":
        return <label>{currResult.toString()}</label>;
      case "bigint":
        return <label>{currResult.toString()}</label>;
      case "string":
        return <label>{currResult}</label>;
      case "object":
        if (Array.isArray(currResult)) {
          return (
            <>
              <label>[</label>
              <div style={{ paddingLeft: "5px" }}>
                {currResult.map((value, index) => (
                  <span key={index.toString()}>{renderResult(value)}</span>
                ))}
              </div>
              <label>]</label>
            </>
          );
        } else {
          return (
            <>
              <label>{"{"}</label>
              <div style={{ paddingLeft: "5px" }}>
                {Object.entries(currResult).map(([key, value], index) => {
                  return (
                    <span key={index.toString()}>
                      <label>{key}:</label>
                      {renderResult(value)}
                    </span>
                  );
                })}
              </div>
              <label>{"}"}</label>
            </>
          );
        }
    }
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
              setLastResult(undefined);
            } else {
              setErrorMessage(
                res.error
                  ? `${res.error.kind}: ${res.error.msg}`
                  : "Unknown error"
              );
              setLastResult(undefined);
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
              setLastResult(result.result);
              setErrorMessage("");
            } else {
              setErrorMessage(
                result.error
                  ? `${result.error.kind}: ${result.error.msg}`
                  : "Unknown error"
              );
            }
          }}
        >
          Eval
        </button>
      </div>
      <label>{errorMessage}</label>
      <div style={{ marginTop: "40px" }}>{generateParams()}</div>
      {lastResult && (
        <>
          <label key="label">Result:</label>
          <div key="result" style={{ paddingLeft: "5px" }}>
            {renderResult(lastResult)}
          </div>
        </>
      )}
    </div>
  );
}

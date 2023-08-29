import * as React from "react";
import "./CelComponent.css";

import init, { cel_eval, cel_details, CelFloat } from "rscel";
import { useState, useEffect } from "react";

export default function CelComponent() {
  const [isInit, setIsInit] = useState<boolean>(false);
  const [errorMessage, setErrorMessage] = useState<string>("");
  const [prog, setProg] = useState<string>("");
  const [params, setParams] = useState<string[]>([]);
  const [paramVals, setParamVals] = useState<any>({});

  useEffect(() => {
    init().then((_res: any) => {
      setIsInit(true);
    });
  }, []);

  if (!isInit) {
    return <div>Loading...</div>;
  }

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
                  const floatval = new CelFloat(Number(event.target.value));

                  newObj[val] = floatval;
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
              const details = res.result.get("details");
              setParams(details.get("params"));
              setErrorMessage("");
            } else {
              setErrorMessage(`${res.error.kind}: ${res.error.msg}`);
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
              setErrorMessage(`Result: ${result.result.toString()}`);
            } else {
              setErrorMessage(`${result.error.kind}: ${result.error.msg}`);
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

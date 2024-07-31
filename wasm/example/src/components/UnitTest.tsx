import * as React from "react";
import { useEffect, useState } from "react";
import { CelEvalResult, CelValue } from "rscel";

export interface UnitTestProps {
  name: string;
  test: () => CelEvalResult;
  expected: CelValue;
}

export const UnitTest = (props: UnitTestProps) => {
  const { name, test, expected } = props;

  const [testPassed, setTestPassed] = useState<boolean>(false);
  const [result, setResult] = useState<any>({});

  const EPSILON = 0.0000001;

  useEffect(() => {
    const test_result = test();

    if (test_result.isSuccess()) {
      const val = test_result.result();
      if (typeof val === "number" && typeof expected === "number") {
        setResult(val);
        setTestPassed(Math.abs(val - expected) < EPSILON);
      } else {
        setTestPassed(val === expected);
      }
    } else {
      setTestPassed(false);
    }
  }, []);

  const renderPassFail = () => {
    if (testPassed) {
      return <label style={{ color: "green" }}>PASS</label>;
    }

    return (
      <span>
        <label style={{ color: "red" }}>FAIL</label>
        <label>{`${result} != ${expected}`}</label>
      </span>
    );
  };

  return (
    <div style={{ display: "grid", gridTemplateColumns: "30% auto" }}>
      <label style={{ wordWrap: "normal" }}>{name}</label>
      {renderPassFail()}
    </div>
  );
};

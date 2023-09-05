import * as React from "react";
import { useEffect, useState } from "react";

export interface UnitTestProps {
  name: string;
  test: () => any;
  expected: any;
}

export const UnitTest = (props: UnitTestProps) => {
  const { name, test, expected } = props;

  const [testPassed, setTestPassed] = useState<boolean>(false);
  const [result, setResult] = useState<any>({});

  const EPSILON = 0.0000001;

  useEffect(() => {
    const result = test();

    if (result.success) {
      if (typeof result.result === "number" && typeof expected === "number") {
        setResult(result.result);
        setTestPassed(Math.abs(result.result - expected) < EPSILON);
      } else {
        setTestPassed(result.result === expected);
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

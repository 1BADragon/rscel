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

  useEffect(() => {
    const result = test();

    console.log(`test-${name}: ${result.toString()}, ${expected.toString()}`);
    setTestPassed(test() === expected);
  }, []);

  const renderPassFail = () => {
    if (testPassed) {
      return <label style={{ color: "green" }}>PASS</label>;
    }

    return <label style={{ color: "red" }}>FAIL</label>;
  };

  return (
    <div style={{ display: "grid", gridTemplateColumns: "30% auto" }}>
      <label style={{ wordWrap: "normal" }}>{name}</label>
      {renderPassFail()}
    </div>
  );
};

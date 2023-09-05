import * as React from "react";

import { cel_eval } from "rscel";
import { UnitTest } from "./UnitTest";

export const UnitTestsComponent = () => {
  const tests: [[string, () => any, any]] = [
    ["basic", () => cel_eval("3+3", {}), 6n],
  ];

  return (
    <div
      style={{
        width: "100%",
      }}
    >
      {tests.map((value) => (
        <UnitTest
          key={`unittest-${value[0]}`}
          name={value[0]}
          test={value[1]}
          expected={value[2]}
        />
      ))}
    </div>
  );
};

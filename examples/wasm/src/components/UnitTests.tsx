import * as React from "react";

import { UnitTest } from "./UnitTest";
import { tests } from "./testCases";

export const UnitTestsComponent = () => {
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

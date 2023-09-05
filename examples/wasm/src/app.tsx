import init from "rscel";
import CelComponent from "./components/CelComponent";
import * as React from "react";
import { UnitTestsComponent } from "./components/UnitTests";
import { useEffect, useState } from "react";

export default function App() {
  const [isInit, setIsInit] = useState<boolean>(false);

  useEffect(() => {
    init().then((_res: any) => {
      setIsInit(true);
    });
  }, []);

  if (!isInit) {
    return <div>Loading...</div>;
  }
  return (
    <div style={{ display: "grid", gridTemplateColumns: "30% auto" }}>
      <UnitTestsComponent />
      <div style={{ height: "640px", width: "480px", marginLeft: "50px" }}>
        <CelComponent />
      </div>
    </div>
  );
}

import CelComponent from "./pages/CelComponent";
import { Card } from "@fluentui/react-components";
import * as React from "react";

export default function App() {
  return (
    <Card style={{ height: "640px", width: "480px", margin: "auto" }}>
      <CelComponent />
    </Card>
  );
}

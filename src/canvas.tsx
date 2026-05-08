import React from "react";
import ReactDOM from "react-dom/client";
import { DrawingCanvas } from "./components/Drawing/DrawingCanvas";

ReactDOM.createRoot(document.getElementById("canvas-root")!).render(
  <React.StrictMode>
    <DrawingCanvas />
  </React.StrictMode>
);

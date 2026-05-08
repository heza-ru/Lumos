import { Toolbar } from "./components/Toolbar/Toolbar";

export default function App() {
  return (
    <div
      style={{
        width: "100vw",
        height: "100vh",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        background: "transparent",
        overflow: "hidden",
      }}
    >
      <Toolbar />
    </div>
  );
}

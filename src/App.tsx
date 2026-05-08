import { Toolbar } from "./components/Toolbar/Toolbar";
import { useToolbarPosition } from "./hooks/useToolbarPosition";

export default function App() {
  const { savePosition } = useToolbarPosition();

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
      onMouseUp={savePosition}
    >
      <Toolbar />
    </div>
  );
}

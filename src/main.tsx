import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";
import { applyStoredTheme } from "@/lib/theme";

// Apply theme synchronously before first paint to avoid flash.
applyStoredTheme().catch(() => {
  // best-effort; if the store isn't ready we'll re-apply once App mounts
});

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);

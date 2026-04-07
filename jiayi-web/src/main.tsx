import React, { useState, useEffect } from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App";
import AppGuide from "./pages/AppGuide";
import "./index.css";

function Router() {
  const [path, setPath] = useState(window.location.pathname);

  useEffect(() => {
    const onPopState = () => setPath(window.location.pathname);
    window.addEventListener("popstate", onPopState);
    return () => window.removeEventListener("popstate", onPopState);
  }, []);

  if (path === "/app-guide") return <AppGuide />;
  return <App />;
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Router />
  </React.StrictMode>
);

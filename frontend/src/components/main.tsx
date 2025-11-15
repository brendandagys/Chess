import { createRoot } from "react-dom/client";

import { App } from "@src/components/App.tsx";
import { NavProvider } from "@src/context/navContext.tsx";

import "@src/css/index.css";

// eslint-disable-next-line @typescript-eslint/no-non-null-assertion
createRoot(document.getElementById("root")!).render(
  <NavProvider>
    <App />
  </NavProvider>
);

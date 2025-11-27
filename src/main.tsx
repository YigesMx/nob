import React from "react";
import ReactDOM from "react-dom/client";

import App from "./App";
import { QueryClientWrapper, ThemeProvider, ThemeSync } from "./app/providers";
import { getThemePreference } from "@/features/settings/api/theme.api";

const initialTheme = await getThemePreference().catch(() => "system" as const);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <QueryClientWrapper>
      <ThemeProvider defaultTheme={initialTheme} storageKey="nob-theme">
        <ThemeSync />
        <App />
      </ThemeProvider>
    </QueryClientWrapper>
  </React.StrictMode>,
);

import { useEffect } from "react";
import { useTheme } from "next-themes";
import { listen } from "@tauri-apps/api/event";

export function ThemeSync() {
  const { setTheme } = useTheme();

  useEffect(() => {
    const unlisten = listen<string>("theme-changed", (event) => {
      console.log("[ThemeSync] Received theme change:", event.payload);
      setTheme(event.payload);
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, [setTheme]);

  return null;
}

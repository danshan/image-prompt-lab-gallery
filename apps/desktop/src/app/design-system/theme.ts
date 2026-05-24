import { useEffect, useState } from "react";

export type ThemePreference = "light" | "dark";

const storageKey = "imglab.theme";

export function normalizeThemePreference(value: string | null | undefined, prefersDark = false): ThemePreference {
  if (value === "light" || value === "dark") {
    return value;
  }
  return prefersDark ? "dark" : "light";
}

export function nextThemePreference(theme: ThemePreference): ThemePreference {
  return theme === "dark" ? "light" : "dark";
}

function readInitialTheme(): ThemePreference {
  if (typeof window === "undefined") {
    return "light";
  }
  return normalizeThemePreference(
    window.localStorage.getItem(storageKey),
    window.matchMedia?.("(prefers-color-scheme: dark)").matches,
  );
}

export function useThemePreference() {
  const [theme, setTheme] = useState<ThemePreference>(() => readInitialTheme());

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    window.localStorage.setItem(storageKey, theme);
  }, [theme]);

  const toggleTheme = () => setTheme(nextThemePreference);

  return { theme, setTheme, toggleTheme };
}

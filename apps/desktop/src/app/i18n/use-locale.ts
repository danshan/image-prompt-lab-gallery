import { useEffect, useMemo, useState } from "react";
import { dictionaries, type Dictionary, type Locale } from "./dictionaries.js";

const storageKey = "imglab.locale";

export function normalizeLocale(value: string | null | undefined, browserLanguage = "en"): Locale {
  if (value === "en" || value === "zh-CN") {
    return value;
  }
  return browserLanguage.toLowerCase().startsWith("zh") ? "zh-CN" : "en";
}

export function nextLocale(locale: Locale): Locale {
  return locale === "en" ? "zh-CN" : "en";
}

function readInitialLocale(): Locale {
  if (typeof window === "undefined") {
    return "en";
  }
  return normalizeLocale(window.localStorage.getItem(storageKey), window.navigator.language);
}

export function useLocalePreference(): {
  locale: Locale;
  dictionary: Dictionary;
  toggleLocale: () => void;
} {
  const [locale, setLocale] = useState<Locale>(() => readInitialLocale());

  useEffect(() => {
    document.documentElement.lang = locale;
    window.localStorage.setItem(storageKey, locale);
  }, [locale]);

  const dictionary = useMemo(() => dictionaries[locale], [locale]);
  const toggleLocale = () => setLocale(nextLocale);

  return { locale, dictionary, toggleLocale };
}

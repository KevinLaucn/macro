import {
  createContext,
  createEffect,
  createSignal,
  useContext,
  type JSX,
  type ParentComponent,
} from "solid-js";
import zhCN from "./locales/zh-CN.json";

export type SupportedLocale = "zh-CN" | "en-US";

const STORAGE_KEY = "macro_locale";

function getDefaultLocale(): SupportedLocale {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved === "zh-CN" || saved === "en-US") {
      return saved;
    }
    if (typeof navigator !== "undefined" && navigator.language?.startsWith("zh")) {
      return "zh-CN";
    }
  } catch {
    // ignore
  }
  return "en-US";
}

export const [locale, setLocaleSignal] = createSignal<SupportedLocale>(getDefaultLocale());

export function setLocale(newLocale: SupportedLocale) {
  setLocaleSignal(newLocale);
  try {
    localStorage.setItem(STORAGE_KEY, newLocale);
    document.documentElement.lang = newLocale;
  } catch {
    // ignore
  }
}

const dictionaries: Record<SupportedLocale, Record<string, string>> = {
  "zh-CN": zhCN as Record<string, string>,
  "en-US": {},
};

/**
 * Global reactive translation helper.
 * Reading locale() establishes a SolidJS reactive dependency in JSX computations.
 */
export function __t(key: string, contextKey?: string, fallback?: string): string {
  if (typeof key !== "string" || !key) return key as unknown as string;
  const current = locale();
  if (current === "zh-CN") {
    const dict = dictionaries["zh-CN"];
    if (contextKey && dict[`${key}@@${contextKey}`]) {
      return dict[`${key}@@${contextKey}`];
    }
    if (dict[key]) {
      return dict[key];
    }
  }
  return fallback ?? key;
}

export const t = __t;

// Attach to global window for AST-injected calls
if (typeof window !== "undefined") {
  (window as unknown as { __t: typeof __t }).__t = __t;
}

interface I18nContextValue {
  locale: () => SupportedLocale;
  setLocale: (l: SupportedLocale) => void;
  t: typeof __t;
}

const I18nContext = createContext<I18nContextValue>({
  locale,
  setLocale,
  t: __t,
});

export const I18nProvider: ParentComponent = (props) => {
  createEffect(() => {
    const current = locale();
    try {
      document.documentElement.lang = current;
    } catch {
      // ignore
    }
  });

  return (
    <I18nContext.Provider value={{ locale, setLocale, t: __t }}>
      {props.children}
    </I18nContext.Provider>
  );
};

export function useI18n() {
  return useContext(I18nContext);
}

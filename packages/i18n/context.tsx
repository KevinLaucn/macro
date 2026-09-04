import {
  createContext,
  createEffect,
  useContext,
  type ParentComponent,
} from "solid-js";
import {
  locale,
  setLocale,
  t,
  __t,
  formatDate,
  formatTime,
  formatDateTime,
  formatBoolean,
  formatNumber,
  type SupportedLocale,
  type InterpolationParams,
  type TOptions,
} from "./runtime";

export {
  locale,
  setLocale,
  t,
  __t,
  formatDate,
  formatTime,
  formatDateTime,
  formatBoolean,
  formatNumber,
  type SupportedLocale,
  type InterpolationParams,
  type TOptions,
};

interface I18nContextValue {
  locale: () => SupportedLocale;
  setLocale: (l: SupportedLocale) => void;
  t: typeof t;
}

const I18nContext = createContext<I18nContextValue>({
  locale,
  setLocale,
  t,
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
    <I18nContext.Provider value={{ locale, setLocale, t }}>
      {props.children}
    </I18nContext.Provider>
  );
};

export function useI18n() {
  return useContext(I18nContext);
}

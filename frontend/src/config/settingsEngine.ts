import { reactive } from "vue";
import { NON_STORE_TYPES, type SettingsConfiguration, type SettingsFromConfig, type SettingItem } from "@/components/settings/settingsSchema";

export const camelToSnake = (str: string) =>
  str.replace(/([a-z0-9])([A-Z])/g, "$1_$2").toLowerCase();

// Recursively extracts default values from setting items.
function collectDefaults(items: readonly SettingItem[], initialValues: Record<string, any>) {
  for (const item of items) {
    const isStorable = !item.noStoreValue && !NON_STORE_TYPES.includes(item.type as any);
    if (item.id && isStorable && item.defaultValue !== undefined) {
      initialValues[item.id] = item.defaultValue;
    }
    // Custom store entries.
    if (item.storeEntries) {
      for (const [key, value] of Object.entries(item.storeEntries)) {
        initialValues[camelToSnake(key)] = value;
      }
    }
    // Traverse nested setting items in accordion.
    if (item.items) {
      collectDefaults(item.items, initialValues);
    }
    // Traverse nested setting sections in subpages.
    if (item.sections) {
      item.sections.forEach(section => collectDefaults(section.items, initialValues));
    }
  }
}

export interface SettingsStore<Config extends SettingsConfiguration, Context = any> {
  rawValues: Record<string, any>;
  typed: SettingsFromConfig<Config>;
  context?: Context;
}

export function createSettingsStore<Config extends SettingsConfiguration, Context = any>(
  config: Config,
  context?: Context
): SettingsStore<Config, Context> {
  const initialValues: Record<string, any> = {};
  config.tabs.forEach(tab => tab.sections.forEach(section => collectDefaults(section.items, initialValues)));

  const rawValues = reactive(initialValues);
  const typed = new Proxy(rawValues, {
    get: (raw, prop: string) => raw[camelToSnake(prop)],
    set: (raw, prop: string, value: any) => {
      raw[camelToSnake(prop)] = value;
      return true;
    }
  }) as SettingsFromConfig<Config>;

  return { rawValues, typed, context };
}
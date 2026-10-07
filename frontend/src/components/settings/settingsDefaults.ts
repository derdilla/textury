import { NON_STORE_TYPES, type SettingItem, type SettingsConfiguration } from "@/components/settings/settingsSchema";

// Recursively extracts default values from setting items.
export function getDefaultsFromConfig(config: SettingsConfiguration): Record<string, any> {
  const defaults: Record<string, any> = {};

  function processItems(items: readonly SettingItem[]) {
    for (const item of items) {
      const isStorable = !item.noStoreValue && !NON_STORE_TYPES.includes(item.type as any);
      if (item.id && isStorable && item.defaultValue !== undefined) {
        defaults[item.id] = item.defaultValue;
      }
      // Custom store entries.
      if (item.storeEntries) {
        Object.assign(defaults, item.storeEntries);
      }
      // Traverse nested setting items in accordion.
      if (item.items) processItems(item.items);
      // Traverse nested setting sections in subpages.
      if (item.sections) item.sections.forEach(s => processItems(s.items));
    }
  }

  config.tabs.forEach(tab => tab.sections.forEach(section => processItems(section.items)));
  return defaults;
}
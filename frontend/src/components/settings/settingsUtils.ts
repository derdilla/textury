import type { SettingsStore } from "@/config/settingsEngine";
import { NON_STORE_TYPES, type SettingItem } from "@/components/settings/settingsSchema";

export function getItemValue(item: SettingItem, store: SettingsStore<any>): any {
  if (typeof item.getValue === "function") {
    return item.getValue(store);
  }
  if (NON_STORE_TYPES.includes(item.type as any) || item.noStoreValue) {
    return undefined;
  }
  return store.rawValues[item.id];
}

export function setItemValue(item: SettingItem, value: any, store: SettingsStore<any>): void {
  if (typeof item.setValue === "function") {
    item.setValue(value, store);
    return;
  }
  if (NON_STORE_TYPES.includes(item.type as any) || item.noStoreValue) return;
  store.rawValues[item.id] = value;
}

export function isVisible(
  hidden: boolean | ((store: SettingsStore<any>) => boolean) | undefined,
  store: SettingsStore<any>
): boolean {
  if (typeof hidden === "function") {
    return !hidden(store);
  }
  return !hidden;
}

export function isDisabled(
  disabled: boolean | ((store: SettingsStore<any>) => boolean) | undefined,
  store: SettingsStore<any>
): boolean {
  if (typeof disabled === "function") {
    return disabled(store);
  }
  return Boolean(disabled);
}

export function getDisabledReason(
  disabledReason: string | ((store: SettingsStore<any>) => string) | undefined,
  store: SettingsStore<any>
): string {
  if (typeof disabledReason === "function") {
    return disabledReason(store);
  }
  return disabledReason ?? "";
}

export function getVisibleItems(
  items: readonly SettingItem[] | undefined,
  store: SettingsStore<any>
): SettingItem[] {
  if (!items) return [];
  return items.filter((item) => isVisible(item.hidden, store));
}

export function filterAccordionItems(
  item: SettingItem,
  store: SettingsStore<any>
): SettingItem {
  if (item.type !== "accordion") return item;

  const validItems = getVisibleItems(item.items, store).filter(
    (child) => child.type !== "accordion"
  );

  return {
    ...item,
    items: validItems,
  };
}
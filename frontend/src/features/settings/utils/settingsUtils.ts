import type { SettingsInstance } from "@/features/settings/composables/settings";
import { NON_STORE_TYPES, type SettingItem } from "@/features/settings/utils/settingsSchema";

export function getItemValue(item: SettingItem, instance: SettingsInstance<any>): any {
  if (typeof item.getValue === "function") {
    return item.getValue(instance);
  }
  if (NON_STORE_TYPES.includes(item.type as any) || item.noStoreValue) {
    return undefined;
  }
  return instance.rawValues[item.id];
}

export function setItemValue(item: SettingItem, value: any, instance: SettingsInstance<any>): void {
  if (typeof item.setValue === "function") {
    item.setValue(value, instance);
    return;
  }
  if (NON_STORE_TYPES.includes(item.type as any) || item.noStoreValue) return;
  instance.rawValues[item.id] = value;
}

export function isVisible(
  hidden: boolean | ((instance: SettingsInstance<any>) => boolean) | undefined,
  instance: SettingsInstance<any>
): boolean {
  if (typeof hidden === "function") {
    return !hidden(instance);
  }
  return !hidden;
}

export function isDisabled(
  disabled: boolean | ((instance: SettingsInstance<any>) => boolean) | undefined,
  instance: SettingsInstance<any>
): boolean {
  if (typeof disabled === "function") {
    return disabled(instance);
  }
  return Boolean(disabled);
}

export function getDisabledReason(
  disabledReason: string | ((instance: SettingsInstance<any>) => string) | undefined,
  instance: SettingsInstance<any>
): string {
  if (typeof disabledReason === "function") {
    return disabledReason(instance);
  }
  return disabledReason ?? "";
}

export function getVisibleItems(
  items: readonly SettingItem[],
  instance: SettingsInstance<any>
): SettingItem[] {
  if (!items) return [];
  return items.filter((item) => isVisible(item.hidden, instance));
}

export function filterAccordionItems(
  item: SettingItem,
  instance: SettingsInstance<any>
): SettingItem {
  if (item.type !== "accordion") return item;

  const validItems = getVisibleItems(item.items ?? [], instance).filter(
    (child) => child.type !== "accordion"
  );

  return {
    ...item,
    items: validItems,
  };
}

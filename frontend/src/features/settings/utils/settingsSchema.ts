import type { SettingsInstance } from "@/features/settings/composables/settings";
import type { Component } from "vue";

export type SettingItemType =
  | "text"
  | "switch"
  | "select"
  | "slider"
  | "range"
  | "page"
  | "action"
  | "accordion"
  | "textinput"
  | "custom";

export interface SelectOption {
  label: string;
  value: string;
}

export interface SettingItem<Context = any> {
  id: string;
  type: SettingItemType;
  title: string;
  description?: string;
  icon?: string;
  reserveIconSpace?: boolean;
  hidden?: boolean | ((instance: SettingsInstance<any, Context>) => boolean);

  defaultValue?: any;
  noStoreValue?: boolean; // Flag to explicitly deactivate automatic Settings Store Entry.
  getValue?: (instance: SettingsInstance<any, Context>) => any;
  setValue?: (value: any, instance: SettingsInstance<any, Context>) => void;

  storeEntries?: Record<string, any>; // Custom Settings Store Entries.

  disabled?: boolean | ((instance: SettingsInstance<any, Context>) => boolean);
  disabledReason?: string | ((instance: SettingsInstance<any, Context>) => string);

  // Select
  options?: readonly SelectOption[];
  // TextInput
  placeholder?: string;
  inputType?: "text" | "password" | "email" | "number" | "url";

  // Action
  actionStyle?: "icon" | "button";
  actionLabel?: string;
  important?: boolean;
  suffixIcon?: string | null;
  destructive?: boolean;
  onClick?: (instance: SettingsInstance<any, Context>) => void;

  // Modal Action
  modalComponent?: Component;
  modalProps?: Record<string, any> | ((instance: SettingsInstance<any, Context>) => Record<string, any>);

  // Page
  sections?: readonly SettingSection<Context>[];

  // Accordion
  items?: readonly SettingItem<Context>[];

  // Slider & Range
  min?: number;
  max?: number;
  step?: number;
  showValue?: boolean;

  // Custom Row Component
  customComponent?: Component;
  customProps?: Record<string, any> | ((instance: SettingsInstance<any, Context>) => Record<string, any>);
}

export interface SettingSection<Context = any> {
  id: string;
  title?: string;
  hidden?: boolean | ((instance: SettingsInstance<any, Context>) => boolean);
  items: readonly SettingItem<Context>[];
}

export interface SettingTab<Context = any> {
  id: string;
  label: string;
  icon: string;
  hidden?: boolean | ((instance: SettingsInstance<any, Context>) => boolean);
  sections: readonly SettingSection<Context>[];
}

export interface SettingsConfiguration<Context = any> {
  defaultTabId?: string;
  tabs: readonly SettingTab<Context>[];
}

export const NON_STORE_TYPES = ["accordion", "action", "page", "custom"] as const;
type NonStorableItemType = typeof NON_STORE_TYPES[number];

// Flatten individual items by unwrapping subpages and accordions.
type FlattenItem<Item> =
  Item extends { sections: readonly SettingSection[] }
    ? Item | FlattenItem<Item["sections"][number]["items"][number]>
    : Item extends { items: readonly SettingItem[] }
    ? Item | FlattenItem<Item["items"][number]>
    : Item;

// Extract every single item from the schema into a flat union.
type AllConfigItems<Config extends SettingsConfiguration> =
  FlattenItem<Config["tabs"][number]["sections"][number]["items"][number]>;

// Filter out non-storable items and convert IDs to camelCase.
type ToStoreKey<Item extends SettingItem> =
  Item extends { noStoreValue: true } | { type: NonStorableItemType }
    ? never
    : Item["id"];

// Map setting items to corresponding TypeScript value types.
type ToStoreValueType<Item extends SettingItem> = {
  switch: boolean;
  slider: number;
  range: [number, number];
  textinput: string;
  text: string;
  select: string;
  accordion: never;
  action: never;
  page: never;
  custom: never;
}[Item["type"]];

type CustomKeys<Config extends SettingsConfiguration> = AllConfigItems<Config> extends infer Item
  ? Item extends { storeEntries: infer Entries }
    ? keyof Entries & string
    : never
  : never;

type CustomValue<Config extends SettingsConfiguration, TargetKey extends string> = AllConfigItems<Config> extends infer Item
  ? Item extends { storeEntries: infer Entries }
    ? TargetKey extends keyof Entries
      ? Entries[TargetKey]
      : never
    : never
  : never;

// All extracted settings store entry types.
export type SettingsFromConfig<Config extends SettingsConfiguration> = {
  [Item in AllConfigItems<Config> as ToStoreKey<Item>]: ToStoreValueType<Item>;
} & {
  [Key in CustomKeys<Config>]: CustomValue<Config, Key>;
};

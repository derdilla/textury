import type { SettingsConfiguration, SettingsFromConfig } from "@/features/settings/utils/settingsSchema";

export type SettingsKey<Config extends SettingsConfiguration> = keyof SettingsFromConfig<Config> & string;

export interface SettingsAdapter<Config extends SettingsConfiguration> {
  getSetting: <K extends SettingsKey<Config>>(
    key: K
  ) => SettingsFromConfig<Config>[K];

  setSetting: <K extends SettingsKey<Config>>(
    key: K,
    value: SettingsFromConfig<Config>[K]
  ) => Promise<boolean> | boolean;
}

export interface SettingsInstance<Config extends SettingsConfiguration, Context = any> {
  rawValues: Record<string, any>;
  typed: SettingsFromConfig<Config>;
  context?: Context;
}

export function useSettings<Config extends SettingsConfiguration, Context = any>(
  _config: Config,
  adapter: SettingsAdapter<Config>,
  context?: Context
): SettingsInstance<Config, Context> {
  const rawValues = new Proxy({}, {
    get: (_, prop: string) => adapter.getSetting(prop as SettingsKey<Config>),
    set: (_, prop: string, value: any) => {
      adapter.setSetting(prop as SettingsKey<Config>, value);
      return true;
    }
  });

  return {
    rawValues,
    typed: rawValues as SettingsFromConfig<Config>,
    context
  };
}

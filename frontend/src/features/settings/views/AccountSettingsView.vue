<script setup lang="ts">
import SettingsView from "@/features/settings/views/SettingsView.vue";
import { accountSettingsConfig } from "@/features/settings/configs/accountSettingsConfig";
import { useAccountSettings } from "@/features/settings/composables/accountSettings";

import { testSettingsConfig } from "@/features/settings/configs/testSettingsConfig";
import { useTestSettings } from "@/features/settings/composables/testSettings";
</script>

<template>
  <SettingsView :config="accountSettingsConfig" :instance="useAccountSettings()" />
  <!--
  <SettingsView :config="testSettingsConfig" :instance="useTestSettings()" />
  -->
</template>

<style scoped>
</style>

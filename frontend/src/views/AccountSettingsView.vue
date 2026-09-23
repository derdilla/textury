<script setup lang="ts">
import { accountSettingsConfig } from "@/config/accountSettingsConfig";
import SettingsView from "@/components/settings/SettingsView.vue";
import { accountSettingsStore, testSettingsStore } from "@/config/settingsStore";
import { testSettingsConfig } from "@/config/testSettingsConfig";
</script>

<template>
  <SettingsView :config="accountSettingsConfig" :store="accountSettingsStore" />
  <!-- <SettingsView :config="testSettingsConfig" :store="testSettingsStore" /> -->
</template>

<style scoped>
</style>
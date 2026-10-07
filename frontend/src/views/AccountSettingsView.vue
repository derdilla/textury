<script setup lang="ts">
import SettingsView from "@/components/settings/SettingsView.vue";
import { accountSettingsConfig } from "@/config/accountSettingsConfig";
import { useAccountSettings } from "@/composables/accountSettings";

//import { testSettingsConfig } from "@/config/testSettingsConfig";
//import { useTestSettings } from "@/composables/testSettings";
</script>

<template>
  <SettingsView :config="accountSettingsConfig" :instance="useAccountSettings()" />
  <!--
  <SettingsView :config="testSettingsConfig" :instance="useTestSettings()" />
  -->
</template>

<style scoped>
</style>
<script setup lang="ts">
import { onMounted } from "vue";
import { useRouter } from "vue-router";
import { whoami } from "@/services/api/managementApi";
import { accountSettingsConfig } from "@/config/accountSettingsConfig";
import SettingsView from "@/components/settings/SettingsView.vue";
import { accountSettingsStore, testSettingsStore } from "@/config/settingsStore";
import { testSettingsConfig } from "@/config/testSettingsConfig";

const router = useRouter();

onMounted(async () => {
  const user = await whoami();
  if (user === null) {
    await router.push("/login");
    return;
  }
});
</script>

<template>
  <SettingsView :config="accountSettingsConfig" :store="accountSettingsStore" />
  <!-- <SettingsView :config="testSettingsConfig" :store="testSettingsStore" /> -->
</template>

<style scoped>
</style>
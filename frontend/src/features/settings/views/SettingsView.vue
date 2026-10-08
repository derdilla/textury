<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import type { SettingsConfiguration, SettingSection, SettingItem } from "@/features/settings/utils/settingsSchema";
import type { SettingsInstance } from "@/features/settings/composables/settings";
import { isVisible, getVisibleItems, filterAccordionItems } from "@/features/settings/utils/settingsUtils";

import SidebarLayout from "@/shared/components/layouts/SidebarLayout.vue";
import SettingRow from "@/features/settings/components/SettingRow.vue";
import SettingAccordion from "@/features/settings/components/SettingAccordion.vue";

const props = withDefaults(
  defineProps<{
    config: SettingsConfiguration;
    instance: SettingsInstance<any>;
    embedded?: boolean;
    subtitle?: string;
    title?: string;
  }>(),
  {
    embedded: false,
    title: "Settings",
  }
);

const route = useRoute();
const router = useRouter();
const openAccordions = ref<Record<string, boolean>>({});

const visibleTabs = computed(() =>
  (props.config.tabs ?? []).filter((tab) => isVisible(tab.hidden, props.instance))
);

const activeTabId = computed(() => {
  const routeTabId = (route?.params?.tabId as string) || "";
  const validTab = visibleTabs.value.find((t) => t.id === routeTabId);
  if (validTab) return validTab.id;

  const defaultId = props.config.defaultTabId;
  if (defaultId && visibleTabs.value.some((t) => t.id === defaultId)) {
    return defaultId;
  }
  return visibleTabs.value[0]?.id ?? "";
});

const activeTab = computed(() =>
  visibleTabs.value.find((tab) => tab.id === activeTabId.value)
);

// When no tab was given in URL, move to default tab url.
watch(
  () => route?.params?.tabId,
  (tabId) => {
    if (!tabId && activeTabId.value) {
      router.replace(`/settings/${activeTabId.value}`);
    }
  },
  { immediate: true }
);

// Helper to recursively find subpages by ID.
function findPageItem(
  id: string,
  container: {
    sections?: readonly SettingSection[];
    items?: readonly SettingItem[];
  }
): SettingItem | null {
  const itemsToSearch: SettingItem[] = [];
  if (container.sections) {
    for (const sec of container.sections) {
      itemsToSearch.push(...(sec.items as SettingItem[]));
    }
  }
  if (container.items) {
    itemsToSearch.push(...(container.items as SettingItem[]));
  }

  for (const item of itemsToSearch) {
    if (item.id === id) return item;
    const found = findPageItem(id, item);
    if (found) return found;
  }
  return null;
}

// Computed NavStack from URL to display in UI and navigate.
const navStack = computed<SettingItem[]>(() => {
  if (!activeTab.value) return [];

  const rawPageParams = route?.params?.pageId;
  const pageIds = Array.isArray(rawPageParams)
    ? rawPageParams
    : rawPageParams
    ? [rawPageParams]
    : [];

  const stack: SettingItem[] = [];
  let currentContainer: {
    sections?: readonly SettingSection[];
    items?: readonly SettingItem[];
  } = activeTab.value;

  for (const pageId of pageIds) {
    const foundItem = findPageItem(pageId, currentContainer);
    if (foundItem) {
      stack.push(foundItem);
      currentContainer = foundItem;
    } else {
      break;
    }
  }

  return stack;
});

const currentPage = computed<SettingItem | null>(
  () => navStack.value.at(-1) ?? null
);

const activeSections = computed<readonly SettingSection[]>(() => {
  const container = (currentPage.value ?? activeTab.value) as any;
  return (container?.sections ?? []).filter((section: SettingSection) =>
    isVisible(section.hidden, props.instance)
  );
});

const currentTitle = computed(
  () => currentPage.value?.title ?? activeTab.value?.label ?? ""
);

const breadcrumbPath = computed(() => {
  if (!activeTab.value || navStack.value.length === 0) return "";
  const parentChain = [
    activeTab.value.label,
    ...navStack.value.slice(0, -1).map((item) => item.title),
  ];
  return parentChain.join(" / ");
});

function selectTab(tabId: string) {
  if (activeTabId.value !== tabId) {
    router.push(`/settings/${tabId}`);
  }
}

function pushPage(pageItem: SettingItem) {
  const currentPathSegments = navStack.value.map((item) => item.id);
  router.push(`/settings/${activeTabId.value}/${[...currentPathSegments, pageItem.id].join("/")}`);
}

function popPage() {
  if (navStack.value.length <= 1) {
    router.push(`/settings/${activeTabId.value}`);
  } else {
    const parentSegments = navStack.value.slice(0, -1).map((item) => item.id);
    router.push(`/settings/${activeTabId.value}/${parentSegments.join("/")}`);
  }
}

function getVisibleSectionItems(section: SettingSection): SettingItem[] {
  return getVisibleItems((section as any).items, props.instance).map((item) =>
    filterAccordionItems(item as any, props.instance)
  );
}

function isPrevAccordionOpen(items: SettingItem[], index: number): boolean {
  if (index <= 0) return false;
  const prevItem = items[index - 1];
  return prevItem?.type === "accordion" && (openAccordions.value[prevItem.id] ?? false);
}
</script>

<template>
  <SidebarLayout :embedded="embedded">
    <!-- Sidebar Header -->
    <template #sidebar-header>
      <button v-if="!embedded" class="square" @click="router.push('/projects')">
        <i>home</i>
      </button>
      <div class="column max title-wrapper">
        <h6 class="bold no-margin sidebar-title">{{ title }}</h6>
        <p v-if="subtitle" class="grey-text no-margin sidebar-subtitle">
          {{ subtitle }}
        </p>
      </div>
    </template>

    <!-- Sidebar Content -->
    <template #sidebar-content>
      <button
        v-for="tab in visibleTabs"
        :key="tab.id"
        class="left-align no-round medium"
        :class="activeTabId === tab.id ? 'secondary-container' : 'transparent'"
        style="padding-left: 0.75rem; padding-right: 0.75rem"
        @click="selectTab(tab.id)"
      >
        <i class="medium" style="padding-right: 0.5rem">{{ tab.icon }}</i>
        <span>{{ tab.label }}</span>
      </button>
    </template>

    <!-- Content Header -->
    <template #content-header>
      <div class="row">
        <Transition name="back-btn">
          <div v-if="navStack.length > 0" class="back-btn-wrapper">
            <button class="square fill back-btn-element" @click="popPage">
              <i>arrow_back</i>
            </button>
          </div>
        </Transition>

        <Transition name="title-shift" mode="out-in">
          <div :key="currentTitle" class="column max title-wrapper">
            <h6 class="margin-none page-title">{{ currentTitle }}</h6>
            <span v-if="breadcrumbPath" class="subtitle">{{ breadcrumbPath }}</span>
          </div>
        </Transition>
      </div>
    </template>

    <!-- Main Body Content -->
    <template #default>
      <template v-for="section in activeSections" :key="section.id">
        <template v-if="getVisibleSectionItems(section).length > 0">
          <h6 v-if="section.title" class="small bold section-title">
            {{ section.title }}
          </h6>

          <div
            class="content-group"
            :class="embedded ? 'surface-container-high' : 'surface-container'"
          >
            <template
              v-for="(item, index) in getVisibleSectionItems(section)"
              :key="item.id"
            >
              <hr v-if="index > 0" />
              <SettingAccordion
                v-if="item.type === 'accordion'"
                :item="item"
                v-model:open="openAccordions[item.id]"
                :has-open-accordion-above="isPrevAccordionOpen(getVisibleSectionItems(section), index)"
                :has-item-below="index < getVisibleSectionItems(section).length - 1"
                :instance="instance"
                @open-page="pushPage"
              />
              <SettingRow
                v-else
                :item="item"
                :is-first-in-group="index === 0"
                :has-open-accordion-above="isPrevAccordionOpen(getVisibleSectionItems(section), index)"
                :instance="instance"
                @open-page="pushPage"
              />
            </template>
          </div>
        </template>
      </template>
    </template>
  </SidebarLayout>
</template>

<style scoped>
.sidebar-title {
  font-weight: 600;
}

.sidebar-subtitle {
  padding-top: 0.3rem;
  font-size: 0.9rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  width: 100%;
}

.title-wrapper {
  display: flex;
  flex-direction: column;
  justify-content: center;
  will-change: transform, opacity;
}

.page-title {
  font-size: 1.1rem;
  line-height: 1.1;
  font-weight: 600;
}

.subtitle {
  font-size: 0.75rem;
  line-height: 1.2;
  opacity: 0.6;
  margin-top: 0.25rem;
}

.back-btn-wrapper {
  width: 2.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  overflow: hidden;
}

.back-btn-element {
  width: 2.5rem;
  height: 2.5rem;
  flex-shrink: 0;
}

/* Animations */
.back-btn-enter-active,
.back-btn-leave-active {
  transition: width 0.22s cubic-bezier(0.2, 0, 0, 1),
              margin-left 0.22s cubic-bezier(0.2, 0, 0, 1);
}
.back-btn-enter-from,
.back-btn-leave-to {
  width: 0;
  margin-left: -1rem;
}

.back-btn-enter-active .back-btn-element,
.back-btn-leave-active .back-btn-element {
  transition: all 0.22s cubic-bezier(0.2, 0, 0, 1);
}

.back-btn-enter-from .back-btn-element,
.back-btn-leave-to .back-btn-element {
  opacity: 0;
  transform: translateX(-10px) scale(0.85);
}

.title-shift-enter-active,
.title-shift-leave-active {
  transition: all 0.18s cubic-bezier(0.2, 0, 0, 1);
}
.title-shift-enter-from {
  opacity: 0;
  transform: translateX(12px);
}
.title-shift-leave-to {
  opacity: 0;
  transform: translateX(-8px);
}

.section-title {
  margin: 0;
  padding-left: 0.5rem;
}

.content-group {
  border-radius: 18px;
  overflow: hidden;
  flex-shrink: 0;
}
</style>

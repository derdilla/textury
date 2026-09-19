<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";

withDefaults(
  defineProps<{
    embedded?: boolean;
    hideSidebar?: boolean;
    contentMaxWidth?: string;
  }>(),
  {
    embedded: false,
    hideSidebar: false,
    contentMaxWidth: "800px",
  }
);

const initialSidebarState = typeof window !== "undefined" && window.innerWidth <= 850;
const isSidebarHidden = ref(initialSidebarState);

let mediaQuery: MediaQueryList | null = null;

function handleMediaChange(e: MediaQueryListEvent | MediaQueryList) {
  isSidebarHidden.value = e.matches;
}

onMounted(() => {
  mediaQuery = window.matchMedia("(max-width: 850px)");
  isSidebarHidden.value = mediaQuery.matches;

  if (mediaQuery.addEventListener) {
    mediaQuery.addEventListener("change", handleMediaChange);
  } else {
    mediaQuery.addListener(handleMediaChange);
  }
});

onUnmounted(() => {
  if (mediaQuery) {
    if (mediaQuery.removeEventListener) {
      mediaQuery.removeEventListener("change", handleMediaChange);
    } else {
      mediaQuery.removeListener(handleMediaChange);
    }
  }
});

function toggleSidebar() {
  isSidebarHidden.value = !isSidebarHidden.value;
}
</script>

<template>
  <div class="layout-container row no-space" :class="{ embedded }">
    <!-- Overlay-Backdrop -->
    <Transition name="fade">
      <div v-if="!hideSidebar && !isSidebarHidden" class="sidebar-backdrop" @click="isSidebarHidden = true"/>
    </Transition>

    <!-- Sidebar -->
    <div v-if="!hideSidebar" class="sidebar"
      :class="[
        embedded ? 'surface-container-high' : 'surface-container',
        { 'is-hidden': isSidebarHidden }
      ]"
    >
      <div class="sidebar-header row align-center padding">
        <slot name="sidebar-header" />
      </div>

      <div class="sidebar-content max list small-scroll-bar padding">
        <slot name="sidebar-content" />
      </div>

      <button class="square transparent close-button" @click="isSidebarHidden = true">
        <i>close</i>
      </button>
    </div>

    <!-- Content Area -->
    <div class="content">
      <!-- Sticky Content Header -->
      <div v-if="!hideSidebar || $slots['content-header']" class="content-header-wrapper">
        <div class="content-header-row row middle-align padding">
          <button v-if="!hideSidebar" class="square fill sidebar-toggle"
            @click="toggleSidebar" :title="isSidebarHidden ? 'Open sidebar' : 'Close sidebar'">
            <i>{{ isSidebarHidden ? 'left_panel_open' : 'left_panel_close' }}</i>
          </button>

          <div class="header-slot-container max">
            <slot name="content-header" />
          </div>
        </div>
      </div>

      <!-- Main Scrollable Content -->
      <div class="content-scrollable small-scroll-bar padding">
        <div class="content-centered list" :style="{ maxWidth: contentMaxWidth }">
          <slot />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.layout-container {
  display: flex;
  align-items: stretch;
  height: 100vh;
  overflow: hidden;
  color-scheme: dark;
  position: relative;
}

.layout-container.embedded {
  height: 100%;
  min-height: 0;
  max-height: 100%;
  width: 100%;
  flex: 1;
}

.layout-container.embedded .sidebar {
  background-color: var(--surface-container-high);
}

.sidebar {
  position: relative;
  width: 300px;
  height: 100%;
  display: flex;
  flex-direction: column;
  border-radius: 0 18px 18px 0;
  box-sizing: border-box;
  flex-shrink: 0;
  transition: margin-left 0.3s cubic-bezier(0.4, 0, 0.2, 1),
              opacity 0.25s ease,
              visibility 0.3s;
  will-change: margin-left, opacity;
  z-index: 10;
}

.sidebar.is-hidden {
  margin-left: -300px;
  opacity: 0;
  visibility: hidden;
  pointer-events: none;
}

.layout-container.embedded .sidebar {
  border-radius: 0;
}

.close-button {
  display: none;
  position: absolute;
  top: 0.75rem;
  right: 0.75rem;
  z-index: 20;
}

.sidebar-header {
  flex-shrink: 0;
}

.sidebar-content {
  border-radius: 0;
  overflow-y: auto;
  flex-grow: 1;
  gap: 0.5rem;
}

.sidebar-backdrop {
  display: none;
}

.content {
  flex-grow: 1;
  flex-shrink: 1;
  min-width: 0;
  min-height: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.header-slot-container {
  flex-grow: 1;
  min-width: 0;
  width: 100%;
}

.content-scrollable {
  flex-grow: 1;
  flex-shrink: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  justify-content: center;
  align-items: flex-start;
}

.content-centered {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding-bottom: 2rem;
  box-sizing: border-box;
}

.small-scroll-bar {
  scrollbar-gutter: stable;
  scrollbar-width: thin;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

@media (max-width: 850px) {
  .sidebar {
    position: fixed;
    top: 0;
    left: 0;
    height: 100%;
    z-index: 100;
    margin-left: 0 !important;
    transform: translateX(0);
    transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1),
                visibility 0.3s;
  }

  .close-button {
    display: inline-flex;
  }

  .sidebar-header {
    padding-right: 2.5rem;
  }

  .sidebar.is-hidden {
    transform: translateX(-100%);
    opacity: 1;
    visibility: hidden;
  }

  .layout-container.embedded .sidebar {
    border-radius: 0 18px 18px 0;
  }

  .sidebar-backdrop {
    display: block;
    position: fixed;
    inset: 0;
    background-color: color-mix(in srgb, rgba(0, 0, 0, 0.3), var(--primary) 10%);
    z-index: 90;
  }

  .dark .sidebar-backdrop {
    background-color: rgba(0, 0, 0, 0.6);
  }

  @media (prefers-reduced-transparency: reduce) {
    .sidebar-backdrop {
      background-color: color-mix(in srgb, var(--surface), #000000 10%) !important;
    }
  }
}
</style>
<script setup lang="ts">
import { computed, ref } from "vue";
import { useEditorLayoutStore } from "@/features/editor/stores/editorLayoutStore";
import { useCompilerOutputStore } from "@/features/editor/stores/compilerOutputStore";
import FloatingElement from "@/shared/components/primitives/FloatingElement.vue";

const editorLayoutStore = useEditorLayoutStore();
const compilerOutputStore = useCompilerOutputStore();

const closeFloating = ref();
const expandFloating = ref();

const isOverlay = computed(() => editorLayoutStore.isRightOverlay.value);
const isLogExpanded = ref(false);

const handleCloseOutput = () => {
  if (closeFloating.value) {
    closeFloating.value.isOpen = false;
  }
  editorLayoutStore.closeOutput();
};

const toggleLogExpansion = () => {
  if (expandFloating.value) {
    expandFloating.value.isOpen = false;
  }
  isLogExpanded.value = !isLogExpanded.value;
};

const statusIcon = computed(() => {
  switch (compilerOutputStore.state.logType) {
    case "ok":
      return "check";
    case "warn":
      return "warning";
    case "err":
      return "error";
    default:
      return "info";
  }
});

const statusLabel = computed(() => {
  switch (compilerOutputStore.state.logType) {
    case "ok":
      return "Successful";
    case "warn":
      return "Warnings";
    case "err":
      return "Errors";
    default:
      return "Idle";
  }
});

const isErrorOrWarn = computed(() => {
  return compilerOutputStore.state.logType === "err" || compilerOutputStore.state.logType === "warn";
});
</script>

<template>
  <div class="panel surface-container">
    <!-- Toolbar -->
    <nav v-if="isOverlay" class="header-container row left-align top-margin left-margin">
      <FloatingElement ref="closeFloating" hover align="right">
        <template #button>
          <button @click="handleCloseOutput" class="square transparent">
            <i>close</i>
          </button>
        </template>
        <template #menu>
          <span class="tooltip bottom active">Close Output</span>
        </template>
      </FloatingElement>
    </nav>

    <main class="main-container padding" :class="{ 'no-tooolbar': !isOverlay, 'expanded': isLogExpanded }">
      <!-- PDF -->
      <div class="embed-wrapper">
        <embed :src="compilerOutputStore.state.lastCompiledUrl"
               type="application/pdf"
               height="100%"
               width="100%">
      </div>

      <!-- Log -->
      <div class="log-section surface-container-high" :class="{ 'expanded': isLogExpanded }">
        <nav class="log-header">
          <div class="status-badge row middle-align">
            <span class="log-title bold">Output Log</span>
            <span class="chip small transparent no-border" :class="{ 'error-text': isErrorOrWarn }">
              <i class="small">{{ statusIcon }}</i>
              <span>{{ statusLabel }}</span>
            </span>
          </div>

          <FloatingElement ref="expandFloating" hover align="left">
            <template #button>
              <button @click="toggleLogExpansion" class="square transparent">
                <i>{{ isLogExpanded ? 'keyboard_arrow_down' : 'keyboard_arrow_up' }}</i>
              </button>
            </template>
            <template #menu>
              <span class="tooltip bottom active">{{ isLogExpanded ? 'Collapse Log' : 'Expand Log' }}</span>
            </template>
          </FloatingElement>
        </nav>

        <div class="log-box">
          <span class="log square no-border scroll">{{ compilerOutputStore.state.log }}</span>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
.panel {
  height: 100vh;
  display: flex;
  flex-direction: column;
  border-radius: 18px 0 0 18px;
  color-scheme: dark;
}

.panel > * {
  border-radius: 0;
}

.header-container {
  height: 2.5rem;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: flex-start;
}

.main-container {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
  gap: 1rem;
  box-sizing: border-box;
  transition: gap 0.25s ease;
}

.main-container.no-tooolbar {
  padding-top: 1rem;
}

.main-container.expanded {
  gap: 0;
}

.embed-wrapper {
  flex: 1;
  min-height: 0;
  height: 100%;
  border-radius: 9px;
  overflow: hidden;
  background-color: var(--surface);
  transition: flex 0.25s cubic-bezier(0.4, 0, 0.2, 1), opacity 0.2s ease, visibility 0.2s ease, margin 0.25s ease;
}

.main-container.expanded .embed-wrapper {
  flex: 0 0 0px;
  opacity: 0;
  visibility: hidden;
  pointer-events: none;
}

.embed-wrapper embed {
  border-radius: 9px;
  width: 100%;
  height: 100%;
}

.log-section {
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  border-radius: 9px;
  overflow: hidden;
  height: 110px;
  transition: flex 0.25s cubic-bezier(0.4, 0, 0.2, 1), height 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.log-section.expanded {
  flex: 1;
  height: auto;
}

.log-header {
  height: 2.5rem;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0;
  margin: 0;
}

.status-badge {
  gap: 0.5rem;
  padding-left: 0.75rem;
}

.log-title {
  font-size: 1rem;
}

.log-box {
  display: flex;
  flex-flow: column;
  flex: 1;
  min-height: 0;
  overflow: hidden;
  padding: 0 0.75rem 0.75rem 0.75rem;
}

.log {
  flex: 1;
  margin: 0;
  overflow: auto;
  font-family: monospace;
  font-size: 0.8rem;
  background: transparent;
}
</style>

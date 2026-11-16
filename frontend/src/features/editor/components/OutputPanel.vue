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

const log = computed(() => (compilerOutputStore.state.log ?? "").trim());

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
    <nav v-if="isOverlay" class="header-container left-align top-margin left-margin">
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

    <main class="main-container" :class="{ 'close-button-toolbar': isOverlay, 'expanded': isLogExpanded, 'no-log': !log }">
      <!-- PDF -->
      <div class="embed-wrapper">
        <iframe :key="compilerOutputStore.state.lastCompiledUrl"
                :src="compilerOutputStore.state.lastCompiledUrl"
                type="application/pdf"
                style="width: 100%; height: 100%; border: none">
        </iframe>
      </div>

      <!-- Log -->
      <Transition name="log">
        <div v-if="log" class="log-section" :class="{ 'expanded': isLogExpanded }">
          <nav class="log-header margin">
            <div class="row middle-align">
              <span class="log-title bold">Output Log</span>
            </div>

            <div class="small-gap row middle-align">
              <span class="chip transparent no-border" :class="{ 'error-text': isErrorOrWarn, 'medium': isLogExpanded }">
                <i class="small">{{ statusIcon }}</i>
                <span>{{ statusLabel }}</span>
              </span>
              <FloatingElement ref="expandFloating" hover align="left">
                <template #button>
                  <button @click="toggleLogExpansion" class="square fill" :class="{ 'small': !isLogExpanded }">
                    <i>{{ isLogExpanded ? 'keyboard_arrow_down' : 'keyboard_arrow_up' }}</i>
                  </button>
                </template>
                <template #menu>
                  <span class="tooltip bottom active">{{ isLogExpanded ? 'Collapse Log' : 'Expand Log' }}</span>
                </template>
              </FloatingElement>
            </div>
          </nav>

          <div class="log-box">
            <pre class="log log-text square no-border">{{ log }}</pre>
          </div>
        </div>
      </Transition>
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
  gap: 0rem;
  box-sizing: border-box;
  transition: gap 0.25s ease;
  padding: 0;
}

.main-container.close-button-toolbar .embed-wrapper {
  border-radius: 0;
  margin-top: 0.75rem;
}

.main-container.expanded {
  gap: 0;
}

.embed-wrapper {
  flex: 1;
  min-height: 0;
  height: 100%;
  border-radius: 18px 0 0 0;
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

.main-container.no-log .embed-wrapper {
  border-radius: 18px 0 0 18px;
}

.embed-wrapper embed {
  border-radius: 18px;
  width: 100%;
  height: 100%;
}

.log-section {
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  overflow: hidden;
  height: 110px;
  transition: flex 0.25s cubic-bezier(0.4, 0, 0.2, 1), height 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  border-radius: 0 0 0 18px;
}

.log-section.expanded {
  flex: 1;
  height: auto;
  border-radius: 18px 0 0 18px;
}

.log-enter-active,
.log-leave-active {
  flex-shrink: 1;
  transition: height 0.25s cubic-bezier(0.4, 0, 0.2, 1), opacity 0.2s ease;
}

.log-enter-from,
.log-leave-to {
  height: 0;
  opacity: 0;
}

.log-header {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0;
  margin: 0;
}

.log-section.expanded .log-header {
  height: 2.5rem;
}

.main-container.expanded.close-button-toolbar .log-header {
  margin-top: 0 !important;
}

.small-gap {
  gap: 0.5rem;
}

.log-title {
  font-size: 1.1rem;
}

.log-box {
  display: flex;
  flex-flow: column;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.log {
  flex: 1;
  margin: 0;
  padding: 0.75rem;
  padding-top: 0;
  overflow: auto;
  background: transparent;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}
</style>

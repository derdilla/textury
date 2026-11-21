<script setup lang="ts">
import { ref } from 'vue'
import FloatingElement from '@/shared/components/primitives/FloatingElement.vue';
import { useEditorLayoutStore } from "@/features/editor/stores/editorLayoutStore";
import { useEditorSession } from "@/features/editor/composables/editorSession";
import ConnectionStatus from '@/features/editor/components/toolbars/ConnectionStatus.vue';
import CompilerControls from '@/features/editor/components/toolbars/CompilerControls.vue';

defineProps<{
  sidebarHidden: boolean;
  outputHidden: boolean;
}>();

const emit = defineEmits<{
  (event: 'toggle-sidebar'): void;
  (event: 'toggle-output'): void;
  (event: 'compile'): void;
}>();

const editorLayoutStore = useEditorLayoutStore();
const editorSession = useEditorSession();

const sidebarFloating = ref();
const outputFloating = ref();

function handleSidebarToggle() {
  if (sidebarFloating.value) {
    sidebarFloating.value.isOpen = false;
  }
  emit('toggle-sidebar');
}

function handleOutputToggle() {
  if (outputFloating.value) {
    outputFloating.value.isOpen = false;
  }
  emit('toggle-output');
}
</script>

<template>
  <div class="header-container no-padding vertical-margin surface row">
    <FloatingElement ref="sidebarFloating" hover align="right">
      <template #button>
        <button v-if="!sidebarHidden" @click="handleSidebarToggle" class="square fill">
          <i>left_panel_close</i>
        </button>
        <button v-if="sidebarHidden" @click="handleSidebarToggle" class="square fill">
          <i>left_panel_open</i>
        </button>
      </template>
      <template #menu>
        <span v-if="sidebarHidden" class="tooltip bottom active">Show Project Panel</span>
        <span v-if="!sidebarHidden" class="tooltip bottom active">Hide Project Panel</span>
      </template>
    </FloatingElement>

    <div class="row scrollable-buttons small-scroll-bar">
      <div v-if="(sidebarHidden || editorLayoutStore.isLeftOverlay.value) && editorSession.openDocument.value != null" class="column max title-wrapper">
        <h6 class="margin-none page-title">{{ editorSession.projectSummary.value?.title }}</h6>
        <span class="subtitle">{{ editorSession.openDocument.value?.wholePath }}</span>
      </div>

      <!--
             <div class="row header-button-bar no-margin no-round surface-container small-scroll-bar">
               <div class="spacer"></div>
               <button class="bar-button no-round small surface-container">
                 <i class="no-margin">format_bold</i>
                 <span>Bold</span>
               </button>
               <button class="bar-button no-round small surface-container">
                 <i class="no-margin">format_italic</i>
                 <span>Italic</span>
               </button>
               <button class="bar-button no-round small surface-container">
                 <i class="no-margin">function</i>
                 <span>Equation</span>
               </button>
               <div class="spacer"></div>
             </div>
      -->

      <ConnectionStatus></ConnectionStatus>
      <div class="max"></div>

      <!-- Lock Icon -->
      <FloatingElement v-if="!editorSession.canEdit.value" hover align="center">
        <template #button>
          <button
            type="button"
            class="square no-round transparent medium disabled-help-btn"
            @click.stop
          >
            <i>lock</i>
          </button>
        </template>
        <template #menu>
          <span class="tooltip bottom active">{{ editorSession.restrictionReason.value }}</span>
        </template>
      </FloatingElement>

      <CompilerControls @compile="emit('compile')" />
    </div>

    <FloatingElement ref="outputFloating" hover align="left">
      <template #button>
        <button v-if="!outputHidden" @click="handleOutputToggle" class="square fill">
          <i>right_panel_close</i>
        </button>
        <button v-if="outputHidden" @click="handleOutputToggle" class="square fill">
          <i>right_panel_open</i>
        </button>
      </template>
      <template #menu>
        <span v-if="outputHidden" class="tooltip bottom active">Show Output Panel</span>
        <span v-if="!outputHidden" class="tooltip bottom active">Hide Output Panel</span>
      </template>
    </FloatingElement>
  </div>
</template>

<style scoped>
.header-container {
  flex-grow: 0 !important;
}

.scrollable-buttons {
  flex-grow: 1;
  flex-shrink: 1;
  overflow-x: auto;
  container-type: inline-size;
}

.disabled-help-btn {
  cursor: help;
}

.header-button-bar {
  justify-content: safe center !important;
  flex-grow: 1;
  gap: 0px;
  overflow: auto;
  flex-shrink: 1;
}

@container (max-width: 12.5rem) {
  .header-button-bar {
    display: none !important;
  }
}

.spacer {
  width: 4px;
  align-self: stretch;
}

.bar-button {
  padding-left: 8px;
  padding-right: 15px;
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
</style>

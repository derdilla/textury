<script setup lang="ts">
import { onMounted, onUnmounted, computed } from 'vue'
import { useEditorLayoutStore } from "@/features/editor/stores/editorLayoutStore";
import { useCompilerOutputStore } from "@/features/editor/stores/compilerOutputStore";
import { useEditorSession } from "@/features/editor/composables/editorSession";
import FileToolbar from '@/features/editor/components/toolbars/FileToolbar.vue';
import CodeEditor from '@/features/editor/components/CodeEditor.vue';

const editorLayoutStore = useEditorLayoutStore();
const compilerOutputStore = useCompilerOutputStore();
const editorSession = useEditorSession();

const isLeftOverlay = computed(() => editorLayoutStore.isLeftOverlay.value);
const isRightOverlay = computed(() => editorLayoutStore.isRightOverlay.value);
const sidebarHidden = computed(() => editorLayoutStore.sidebarHidden.value);
const outputHidden = computed(() => editorLayoutStore.outputHidden.value);

const paddingLeft = computed(() => (isLeftOverlay.value || sidebarHidden.value) ? '1rem' : '0px');
const paddingRight = computed(() => (isRightOverlay.value || outputHidden.value) ? '1rem' : '0px');

function handleSidebarToggle() {
  editorLayoutStore.toggleSidebarByUser();
}

function handleOutputToggle() {
  editorLayoutStore.toggleOutputByUser();
}

async function compile() {
  if (!editorSession.canCompile.value) return;
  let compilerType = compilerOutputStore.state.compilerPref;
  if (!compilerType) {
    if (editorSession.openDocument.value?.wholePath?.endsWith("typ")) {
      compilerType = "Typst";
    } else {
      compilerType = "PdfLatex";
    }
  }
  editorSession.compile(compilerType);
}

const handleSaveShortcut = (event: KeyboardEvent) => {
  if ((event.metaKey || event.ctrlKey) && event.key === 's') {
    event.preventDefault();
    if (editorSession.canCompile.value) {
      compile();
    }
  }
};

onMounted(() => {
  window.addEventListener('keydown', handleSaveShortcut);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleSaveShortcut);
});
</script>

<template>
  <div class="panel surface field" :style="{ paddingLeft, paddingRight }">
    <FileToolbar
      :sidebar-hidden="sidebarHidden"
      :output-hidden="outputHidden"
      @toggle-sidebar="handleSidebarToggle"
      @toggle-output="handleOutputToggle"
      @compile="compile"
    />
    <div class="editor-container no-round max bottom-margin small-scroll-bar">
      <CodeEditor v-show="editorSession.openDocument.value !== null" />
      <div v-show="editorSession.openDocument.value === null" class="medium middle center-align">
        <div>
          <i class="extra">docs</i>
          <h5 class="bold responsive-title">No open file</h5>
          <h6 class="small responsive-description">Open a file from the project panel to start editing.</h6>

          <div v-if="sidebarHidden || editorLayoutStore.isLeftOverlay.value" class="empty-actions">
            <button tabindex="0" class="chip medium primary no-border" @click="handleSidebarToggle">
              <i>arrow_insert</i> Open Project Panel
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  height: 100vh;
  min-height: 0;
  color-scheme: dark;
}

.editor-container {
  flex: 1 1 0;
  min-height: 0;
  min-width: 0;
  overflow-y: visible;
}

.editor-container > * {
  border-radius: 0;
}

.small-scroll-bar {
  scrollbar-gutter: stable;
  scrollbar-width: thin;
}

.responsive-title {
  margin-top: 1rem;
  margin-bottom: 0.5rem;
}

.responsive-description {
  margin-top: 0;
  margin-bottom: 0;
}

.empty-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  justify-content: center;
  margin-top: 1.5rem;
}

@media (max-width: 600px) {
  .responsive-title {
    font-size: 1.25rem;
  }

  .responsive-description {
    font-size: 0.8rem;
  }
}
</style>

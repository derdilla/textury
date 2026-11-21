<script setup lang="ts">
import FloatingElement from '@/shared/components/primitives/FloatingElement.vue';
import { useCompilerOutputStore } from "@/features/editor/stores/compilerOutputStore";
import { useEditorSession } from "@/features/editor/composables/editorSession";

const compilerOutputStore = useCompilerOutputStore();
const editorSession = useEditorSession();

const emit = defineEmits<{
  (event: 'compile'): void;
}>();
</script>

<template>
  <nav v-if="editorSession.canCompile.value" class="no-margin group split"
    :style="{ cursor: (!editorSession.openDocument.value || !editorSession.projectId.value) ? 'not-allowed' : 'auto' }">
    <a @click="emit('compile')" :class="{ 'is-disabled': (!editorSession.openDocument.value || !editorSession.projectId.value) }"
      class="no-round compile-button chip medium primary no-border">
      <span>Compile</span>
      <progress class="max" value="0" max="100"></progress>
    </a>
    <FloatingElement align="left">
      <template #button>
        <a tabindex="0" :class="{ 'is-disabled': (!editorSession.openDocument.value || !editorSession.projectId.value) }"
          class="square compile-dropdown-button chip medium primary no-border">
          <i>keyboard_arrow_down</i>
        </a>
      </template>
      <template #menu>
        <menu class="dropdown no-wrap no-round active surface-container">
          <button @click="compilerOutputStore.state.compilerPref = null"
            :class="{ 'secondary-container': compilerOutputStore.state.compilerPref === null }"
            class="transparent no-round left-align">
            <i class="small">auto_awesome</i> Default
          </button>
          <li><hr></li>
          <button @click="compilerOutputStore.state.compilerPref = 'PdfLatex'"
            :class="{ 'secondary-container': compilerOutputStore.state.compilerPref === 'PdfLatex' }"
            class="transparent no-round left-align">
            <i class="small">picture_as_pdf</i> PDFLatex
          </button>
          <button @click="compilerOutputStore.state.compilerPref = 'XeLatex'"
            :class="{ 'secondary-container': compilerOutputStore.state.compilerPref === 'XeLatex' }"
            class="transparent no-round left-align">
            <i class="small">translate</i> XeLatex
          </button>
          <button @click="compilerOutputStore.state.compilerPref = 'LuaLatex'"
            :class="{ 'secondary-container': compilerOutputStore.state.compilerPref === 'LuaLatex' }"
            class="transparent no-round left-align">
            <i class="small">code</i> LuaLatex
          </button>
          <li><hr></li>
          <button @click="compilerOutputStore.state.compilerPref = 'Typst'"
            :class="{ 'secondary-container': compilerOutputStore.state.compilerPref === 'Typst' }"
            class="transparent no-round left-align">
            <i class="small">bolt</i> Typst
          </button>
        </menu>
      </template>
    </FloatingElement>
  </nav>
</template>

<style scoped>
.compile-button {
  border-top-right-radius: 0 !important;
  border-bottom-right-radius: 0 !important;
}

.compile-dropdown-button {
  border-top-left-radius: 0 !important;
  border-bottom-left-radius: 0 !important;
}

.is-disabled {
  pointer-events: none !important;
  opacity: 0.5 !important;
  cursor: not-allowed !important;
}

.dropdown hr {
  margin: 3px;
}

.dropdown i {
  margin-right: 0.5rem;
}

.dropdown {
  gap: 4px;
  box-shadow: none !important;
}
</style>

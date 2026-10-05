<script setup lang="ts">
import { yCollab } from 'y-codemirror.next'

import {
  keymap, highlightSpecialChars, drawSelection, highlightActiveLine, dropCursor,
  rectangularSelection, crosshairCursor,
  lineNumbers, highlightActiveLineGutter, EditorView
} from "@codemirror/view"
import { EditorState } from "@codemirror/state";
import { defaultHighlightStyle, syntaxHighlighting, indentOnInput, bracketMatching, foldGutter, foldKeymap } from "@codemirror/language"
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands"
import { searchKeymap, highlightSelectionMatches } from "@codemirror/search"
import { autocompletion, completionKeymap, closeBrackets, closeBracketsKeymap } from "@codemirror/autocomplete"
import { lintKeymap } from "@codemirror/lint"
import { latex } from 'codemirror-lang-latex';
import { typst } from 'codemirror-lang-typst';
import * as random from 'lib0/random'
import * as Y from 'yjs';

import { onMounted, onUnmounted, ref, watch } from 'vue'
import { oneDark } from "@codemirror/theme-one-dark";
import FloatingElement from '@/components/general/FloatingElement.vue';
import { useAuthStore } from '@/stores/auth';

import { useEditorLayoutStore } from "@/stores/editorLayout";
import { useCompilerOutputStore } from "@/stores/compilerOutput";
import { useEditorSession } from "@/composables/editorSession";

const editorLayoutStore = useEditorLayoutStore();
const compilerOutputStore = useCompilerOutputStore();
const editorSession = useEditorSession();
const authStore = useAuthStore();

const usercolors = [
  { color: '#30bced', light: '#30bced33' },
  { color: '#6eeb83', light: '#6eeb8333' },
  { color: '#ffbc42', light: '#ffbc4233' },
  { color: '#ecd444', light: '#ecd44433' },
  { color: '#ee6352', light: '#ee635233' },
  { color: '#9ac2c9', light: '#9ac2c933' },
  { color: '#8acb88', light: '#8acb8833' },
  { color: '#1be7ff', light: '#1be7ff33' }
]

const editorDiv = ref();

let editor: EditorView | null = null;
function setupEditor() {
  if (editor) {
    editor.destroy();
    editor = null;
  }

  // select a random color for this user

  if (!editorSession.projectId.value) {
    console.info(`No open project`);
    return;
  }

  const yText = editorSession.openDocument.value?.document;
  const awareness = editorSession.awareness.value;

  if (yText == null || awareness == null) {
    console.warn(`No open document: ${yText}, ${awareness}`);
    return;
  }

  const undoManager = new Y.UndoManager(yText);

  // Not the real id format; We just need any user specific integer.
  const uid = authStore.state.user?.id;
  let idNum = random.uint32();
  if (uid) {
    idNum = uid.charCodeAt(2) + uid.charCodeAt(3) + uid.charCodeAt(4);
  }
  const userColor = usercolors[idNum % usercolors.length]!;
  awareness.setLocalStateField('user', {
    name: authStore.state.user?.username ?? 'Anonymous ' + Math.floor(Math.random() * 100),
    color: userColor.color,
    colorLight: userColor.light
  });

  const exts = [
    // basicSetup:
    lineNumbers(),
    highlightActiveLineGutter(),
    highlightSpecialChars(),
    history(),
    foldGutter(),
    drawSelection(),
    dropCursor(),
    EditorState.allowMultipleSelections.of(true),
    indentOnInput(),
    syntaxHighlighting(defaultHighlightStyle, {fallback: true}),
    bracketMatching(),
    closeBrackets(),
    autocompletion(),
    rectangularSelection(),
    crosshairCursor(),
    highlightActiveLine(),
    highlightSelectionMatches(),
    keymap.of([
      ...closeBracketsKeymap,
      ...defaultKeymap,
      ...searchKeymap,
      ...historyKeymap,
      ...foldKeymap,
      ...completionKeymap,
      ...lintKeymap,
    ]),

    // Custom:
    // TODO: show message for Esc+Tab escape
    keymap.of([indentWithTab]),
    oneDark,
    EditorView.lineWrapping,
    yCollab(yText, awareness, { undoManager })
  ];

  if (editorSession.openDocument.value?.wholePath?.endsWith("typ")) {
    exts.push(typst());
  } else {
    exts.push(latex());
  }

  const state = EditorState.create({
    doc: yText.toString(),
    extensions: exts,
  });

  editor = new EditorView({
    state,
    parent: editorDiv.value,
  });
  editor.requestMeasure();
}

async function compile() {
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
    compile();
  }
};

onMounted(() => {
  setupEditor();
  window.addEventListener('keydown', handleSaveShortcut);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleSaveShortcut);
});

watch(
  () => editorSession.openDocument.value,
  () => {
    setupEditor();
  }
);
</script>

<template>
  <div class="panel surface field" :style="{
    paddingLeft: editorLayoutStore.sidebarHidden.value ? '15px' : '0px',
    paddingRight: editorLayoutStore.outputHidden.value ? '15px' : '0px'
  }">
    <div class="header-container no-padding vertical-margin surface row">
      <button v-if="!editorLayoutStore.sidebarHidden.value" @click="editorLayoutStore.toggleSidebar" class="square fill">
        <i>left_panel_close</i>
      </button>
      <button v-if="editorLayoutStore.sidebarHidden.value" @click="editorLayoutStore.toggleSidebar" class="square fill">
        <i>left_panel_open</i>
      </button>

      <div class="row scrollable-buttons small-scroll-bar">

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

        <nav class="no-margin group split"
          :style="{ cursor: (!editorSession.openDocument || !editorSession.projectId.value) ? 'not-allowed' : 'auto' }">
          <a @click="compile" :class="{ 'is-disabled': (!editorSession.openDocument || !editorSession.projectId.value) }"
            class="no-round compile-button chip medium primary no-border">
            <span>Compile</span>
            <progress class="max" value="0" max="100"></progress>
          </a>
          <FloatingElement align="left">
            <template #button>
              <a tabindex="0" :class="{ 'is-disabled': (!editorSession.openDocument || !editorSession.projectId.value) }"
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

        <FloatingElement hover align="center">
          <template #button>
            <button @click="editorLayoutStore.toggleLog" class="square"
              :class="{ 'fill': !editorLayoutStore.showLog.value, 'border': editorLayoutStore.showLog.value }">
              <i v-if="compilerOutputStore.state.logType === 'ok'">check</i>
              <i v-if="compilerOutputStore.state.logType === 'warn'">warning</i>
              <i v-if="compilerOutputStore.state.logType === 'err'">error</i>
            </button>
          </template>
          <template #menu>
            <span class="tooltip bottom active">Compilation Log</span>
          </template>
        </FloatingElement>

      </div>

      <button v-if="!editorLayoutStore.outputHidden.value" @click="editorLayoutStore.toggleOutput" class="square fill">
        <i>right_panel_close</i>
      </button>
      <button v-if="editorLayoutStore.outputHidden.value" @click="editorLayoutStore.toggleOutput" class="square fill">
        <i>right_panel_open</i>
      </button>
    </div>
    <div class="editor-container max bottom-margin small-scroll-bar">
      <div v-show="editorSession.openDocument !== null" class="editor" ref="editorDiv"></div>
      <div v-show="editorSession.openDocument === null" class="medium middle center-align">
        <div>
          <i class="extra">docs</i>
          <h5 class="bold">No open file</h5>
          <h6 class="small">Open a file from the file browser to start editing.</h6>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.panel {
  height: 100vh;
  color-scheme: dark;
}

.header-container {
  flex-grow: 0 !important;
}

.scrollable-buttons {
  flex-grow: 1;
  flex-shrink: 1;
  overflow-x: auto;
  container-type: inline-size;
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
}

.editor-container {
  border-radius: 9px;
  flex-grow: 1;
  overflow: auto;
}

.editor-container>* {
  border-radius: 0;
}

.small-scroll-bar {
  scrollbar-gutter: stable;
  scrollbar-width: thin;
}

:deep(.cm-editor) {
  background-color: var(--surface-container);
  clip-path: inset(0px round 9px);
}

:deep(.cm-gutters) {
  background-color: var(--surface-container);
  color: white;
  opacity: 0.5;
  border: none;
}
.dropdown {
  box-shadow: none !important;
}
</style>

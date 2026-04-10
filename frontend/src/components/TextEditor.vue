<script setup lang="ts">

import * as Y from 'yjs'
import { yCollab } from 'y-codemirror.next'
import { WebsocketProvider } from 'y-websocket'

import { basicSetup, EditorView } from "codemirror";
import { EditorState } from "@codemirror/state";
import { latex } from 'codemirror-lang-latex';
import { typst } from 'codemirror-lang-typst';
import * as random from 'lib0/random'

import { onMounted, onUnmounted, ref, watch } from 'vue'
import { oneDark } from "@codemirror/theme-one-dark";
import { editCtrl } from "@/services/editorController.ts";
import { ProjectApi } from "@/services/projectApi.ts";
import FloatingElement from './FloatingElement.vue';

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
let wsProvider: WebsocketProvider | null = null;
function setupEditor(openFile: string | null) {
  if (editor) {
    editor.destroy();
    editor = null;
  }
  if (wsProvider) {
    wsProvider.destroy();
    wsProvider = null;
  }

  // select a random color for this user
  const userColor = usercolors[random.uint32() % usercolors.length]!; // TODO: username based

  const ydoc = new Y.Doc();

  if (!openFile || !editCtrl.openProject) {
    console.info(`No open file in ${openFile}, ${editCtrl.openProject}`);
    return;
  }
  const api = new ProjectApi(editCtrl.openProject);
  const wsUrl = api.ySocketUrl();
  wsProvider = new WebsocketProvider(wsUrl, openFile.substr(1), ydoc, { connect: true });

  const ytext = ydoc.getText('codemirror');
  const undoManager = new Y.UndoManager(ytext);

  wsProvider.awareness.setLocalStateField('user', {
    name: 'Anonymous ' + Math.floor(Math.random() * 100),
    color: userColor.color,
    colorLight: userColor.light
  });

  const state = EditorState.create({
    doc: ytext.toString(),
    extensions: [
      basicSetup,
      oneDark, // TODO: Customizable
      //javascript(),
      typst(),
      latex(),
      yCollab(ytext, wsProvider.awareness, { undoManager })
    ]
  });

  editor = new EditorView({ state, parent: editorDiv.value });
  editor.requestMeasure();
}

async function compile() {
  if (!editCtrl.openFile || !editCtrl.openProject) return;
  const api = new ProjectApi(editCtrl.openProject);
  const res = await api.compile(editCtrl.openFile, { compiler: editCtrl.compilerPref });
  editCtrl.log = res.output?.log ?? null;
  if ((!res.success) || (res.output?.log?.toLowerCase().includes('error') ?? true)) {
    editCtrl.logType = 'err';
  } else if (res.output?.log?.toLowerCase().includes('warn') ?? false) {
    editCtrl.logType = 'warn';
  } else {
    editCtrl.logType = 'ok';
  }
  if (res.output?.document) editCtrl.lastCompiled = res.output?.document
}

const handleSaveShortcut = (event) => {
  if ((event.metaKey || event.ctrlKey) && event.key === 's') {
    event.preventDefault();
    compile();
  }
};

onMounted(() => {
  setupEditor(editCtrl?.openFile ?? null);
  window.addEventListener('keydown', handleSaveShortcut);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleSaveShortcut);
});

watch(
  () => editCtrl?.openFile,
  (newValue) => {
    setupEditor(newValue ?? null);
    console.log("Changed editor file to: " + editCtrl?.openFile);
  }
);

</script>

<template>
  <div class="panel surface field" :style="{
    paddingLeft: editCtrl.sidebarHidden ? '15px' : '0px',
    paddingRight: editCtrl.outputHidden ? '15px' : '0px'
  }">
    <div class="header-container no-padding vertical-margin surface row">
      <button v-if="!editCtrl.sidebarHidden" @click="editCtrl.sidebarHidden = !editCtrl.sidebarHidden"
        class="square fill">
        <i>left_panel_close</i>
      </button>
      <button v-if="editCtrl.sidebarHidden" @click="editCtrl.sidebarHidden = !editCtrl.sidebarHidden"
        class="square fill">
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
          :style="{ cursor: (!editCtrl.openFile || !editCtrl.openProject) ? 'not-allowed' : 'auto' }">
          <a @click="compile" :class="{ 'is-disabled': (!editCtrl.openFile || !editCtrl.openProject) }"
            class="no-round compile-button chip medium primary no-border">
            <span>Compile</span>
            <progress class="max" value="0" max="100"></progress>
          </a>
          <FloatingElement align="left">
            <template #button>
              <a tabindex="0" :class="{ 'is-disabled': (!editCtrl.openFile || !editCtrl.openProject) }"
                class="square compile-dropdown-button chip medium primary no-border">
                <i>keyboard_arrow_down</i>
              </a>
            </template>
            <template #menu>
              <menu id="dropdown" class="no-wrap no-round active surface-container shadow">
                <li @click="editCtrl.compilerPref = null"
                  :class="{ 'secondary-container': editCtrl.compilerPref === null }" class="no-round small left-align">
                  Default</li>
                <li>
                  <hr>
                </li>
                <li @click="editCtrl.compilerPref = 'PdfLatex'"
                  :class="{ 'secondary-container': editCtrl.compilerPref === 'PdfLatex' }"
                  class="no-round small left-align">PDFLatex</li>
                <li @click="editCtrl.compilerPref = 'XeLatex'"
                  :class="{ 'secondary-container': editCtrl.compilerPref === 'XeLatex' }"
                  class="no-round small left-align">XeLatex</li>
                <li @click="editCtrl.compilerPref = 'LuaLatex'"
                  :class="{ 'secondary-container': editCtrl.compilerPref === 'LuaLatex' }"
                  class="no-round small left-align">LuaLatex</li>
                <li>
                  <hr>
                </li>
                <li @click="editCtrl.compilerPref = 'Typst'"
                  :class="{ 'secondary-container': editCtrl.compilerPref === 'Typst' }"
                  class="no-round small left-align">Typst</li>
              </menu>
            </template>
          </FloatingElement>
        </nav>

        <FloatingElement hover align="center">
          <template #button>
            <button @click="editCtrl.showLog = !editCtrl.showLog" class="square"
              :class="{ 'fill': !editCtrl.showLog, 'border': editCtrl.showLog }">
              <i v-if="editCtrl.logType === 'ok'">check</i>
              <i v-if="editCtrl.logType === 'warn'">warning</i>
              <i v-if="editCtrl.logType === 'err'">error</i>
            </button>
          </template>
          <template #menu>
            <span class="tooltip bottom active">Compilation Log</span>
          </template>
        </FloatingElement>

      </div>

      <button v-if="!editCtrl.outputHidden" @click="editCtrl.outputHidden = !editCtrl.outputHidden" class="square fill">
        <i>right_panel_close</i>
      </button>
      <button v-if="editCtrl.outputHidden" @click="editCtrl.outputHidden = !editCtrl.outputHidden" class="square fill">
        <i>right_panel_open</i>
      </button>
    </div>
    <div class="editor-container max bottom-margin small-scroll-bar">
      <div v-show="editCtrl?.openFile !== null" class="editor" ref="editorDiv"></div>
      <div v-show="!(editCtrl?.openFile !== null)" class="center-align middle">
        <i class="extra">docs</i>
        <p class="h5">No open file.</p>
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
  position: sticky;
  top: 0;
  z-index: 2;
  flex-grow: 0;
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

#dropdown li {
  min-height: 0rem;
}

hr {
  margin: 3px;
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
</style>

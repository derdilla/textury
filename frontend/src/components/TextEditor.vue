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
import { editCtrl, Compiler } from "@/services/editorController.ts";
import { ProjectApi } from "@/services/projectApi.ts";

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
  }
);

</script>

<template>
  <div class="panel" :style="{ paddingLeft: editCtrl.sidebarHidden ? '15px' : '0px', 
    paddingRight: editCtrl.outputHidden ? '15px' : '0px'}">
    <div class="header-container">
      <button v-if="!editCtrl.sidebarHidden" @click="editCtrl.sidebarHidden = !editCtrl.sidebarHidden"
        class="square fill no-shrinking">
        <i>left_panel_close</i>
      </button>
      <button v-if="editCtrl.sidebarHidden" @click="editCtrl.sidebarHidden = !editCtrl.sidebarHidden"
        class="square fill no-shrinking">
        <i>left_panel_open</i>
      </button>

      <div class="header-button-bar">
        <button class="bar-button">
          <i>format_bold</i>
          <span>Bold</span>
        </button>
        <button class="bar-button">
          <i>format_italic</i>
          <span>Italic</span>
        </button>
        <button class="bar-button">
          <i>function</i>
          <span>Equation</span>
        </button>
      </div>

      <nav class="no-shrinking compile-group group split">
        <button @click="compile" :disabled="(!editCtrl.openFile || !editCtrl.openProject)" class="no-round">
          <span>Compile</span>
          <progress class="max" value="0" max="100"></progress>
        </button>
        <div tabindex="0">
          <button :disabled="(!editCtrl.openFile || !editCtrl.openProject)" class="square">
            <i>keyboard_arrow_down</i>
          </button>
          <menu class="left no-wrap">
            <li @click="editCtrl.compilerPref = null" :class="{ active: editCtrl.compilerPref === null }">
              <a>default</a>
            </li>
            <li @click="editCtrl.compilerPref = Compiler.typst"
              :class="{ active: editCtrl.compilerPref === Compiler.typst }">
              <a>typst</a>
            </li>
            <li @click="editCtrl.compilerPref = Compiler.pdflatex"
              :class="{ active: editCtrl.compilerPref === Compiler.pdflatex }">
              <a>pdflatex</a>
            </li>
            <li @click="editCtrl.compilerPref = Compiler.xelatex"
              :class="{ active: editCtrl.compilerPref === Compiler.xelatex }">
              <a>xelatex</a>
            </li>
            <li @click="editCtrl.compilerPref = Compiler.lualatex"
              :class="{ active: editCtrl.compilerPref === Compiler.lualatex }">
              <a>lualatex</a>
            </li>
          </menu>
        </div>
      </nav>

      <button @click="editCtrl.showLog = !editCtrl.showLog" class="square fill no-shrinking">
        <i v-if="editCtrl.logType === 'ok'">check</i>
        <i v-if="editCtrl.logType === 'warn'">warning</i>
        <i v-if="editCtrl.logType === 'err'">error</i>
        <span class="tooltip bottom">Compilation Log</span>
      </button>

      <button v-if="!editCtrl.outputHidden" @click="editCtrl.outputHidden = !editCtrl.outputHidden"
        class="square fill no-shrinking">
        <i>right_panel_close</i>
      </button>
      <button v-if="editCtrl.outputHidden" @click="editCtrl.outputHidden = !editCtrl.outputHidden"
        class="square fill no-shrinking">
        <i>right_panel_open</i>
      </button>
    </div>
    <div class="editor-container">
      <div v-show="editCtrl?.openFile !== null" ref="editorDiv"></div>
      <div v-show="!(editCtrl?.openFile !== null)" class="center-align middle">
        <i class="extra">docs</i>
        <p class="h5">No open file.</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.panel {
  background-color: var(--surface);
  height: 100vh;
  display: flex;
  flex-direction: column;
  flex-wrap: nowrap;
  justify-content: flex-start;
  align-items: stretch;
  overflow-y: auto;
  overflow-x: hidden;
  color-scheme: dark;
}

.header-container {
  position: sticky;
  top: 0;
  z-index: 2;
  background-color: var(--surface);
  padding-top: 15px;
  padding-bottom: 15px;
  display: flex;
  flex-wrap: nowrap;
  flex-direction: row;
  justify-content: flex-start;
  align-items: center;
  gap: 15px;
  flex-shrink: 0;
}

.no-shrinking {
  flex-shrink: 0;
}

.header-button-bar {
  background-color: var(--surface-container);
  border-radius: 10px;
  display: flex;
  height: 43px;
  flex-grow: 1;
  gap: 0px;
  flex-wrap: nowrap;
  padding: 5px;
  overflow-y: hidden;
  overflow-x: auto;
}

.bar-button {
  padding-left: 10px;
  padding-right: 15px;
  background-color: var(--surface-container);
  color: white;
  border-radius: 7px;
  height: 33px;
  flex-shrink: 0;
}

.bar-button:hover {
  background-color: var(--surface-container-high);
}

.compile-group {
  margin-block-start: 0px !important;
}

.editor-container {
  background-color: var(--surface);
  border-radius: 12px;
  flex-grow: 1;
  margin-bottom: 15px;
}

</style>

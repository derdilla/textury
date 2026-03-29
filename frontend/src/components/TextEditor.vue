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
  <nav class="max small-padding">
    <button v-if="editCtrl.sidebarHidden" @click="editCtrl.sidebarHidden = !editCtrl.sidebarHidden"
      class="transparent small">
      <i>menu</i>
    </button>

    <!-- placeholder actions -->
    <button class="transparent small">
      <i>format_bold</i>
      <span>Bold</span>
    </button>
    <button class="transparent small">
      <i>format_italic</i>
      <span>Italic</span>
    </button>

    <div class="max"></div>

    <nav class="group split">
      <button @click="compile" :disabled="(!editCtrl.openFile || !editCtrl.openProject)" class="left-round small">
        <span>Compile</span>
        <progress class="max" value="0" max="100"></progress>
      </button>
      <div>
        <button :disabled="(!editCtrl.openFile || !editCtrl.openProject)" class="right-round square small">
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

      <button @click="editCtrl.showLog = !editCtrl.showLog" class="circle transparent small">
        <i v-if="editCtrl.logType === 'ok'">check</i>
        <i v-if="editCtrl.logType === 'warn'">warning</i>
        <i v-if="editCtrl.logType === 'err'">error</i>
        <span class="tooltip bottom">Compilation Log</span>
      </button>
    </nav>
  </nav>

  <div class="editorContainers">
    <div v-show="editCtrl?.openFile !== null" ref="editorDiv"></div>

    <div v-show="!(editCtrl?.openFile !== null)" class="center-align middle">
      <i class="extra">docs</i>
      <p class="h5">No open file.</p>
    </div>

  </div>

</template>

<style scoped>
.CodeMirror {
  height: 100vh;
}

.editorContainers {
  height: 100%;
  contain: size;
  display: block;
  overflow: scroll;
}
</style>

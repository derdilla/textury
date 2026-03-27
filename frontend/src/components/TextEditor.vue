<script setup lang="ts">

import * as Y from 'yjs'
import { yCollab } from 'y-codemirror.next'
import { WebsocketProvider } from 'y-websocket'

import { basicSetup, EditorView } from "codemirror";
import { EditorState } from "@codemirror/state";
//import { javascript } from '@codemirror/lang-javascript'
import * as random from 'lib0/random'

import { onMounted, ref, watch } from 'vue'
import { oneDark } from "@codemirror/theme-one-dark";
import { editCtrl } from "@/services/editorController.ts";
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
  console.info("setupEditor");
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

  //const provider = new WebrtcProvider('codemirror6-demo-room', ydoc)
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
      yCollab(ytext, wsProvider.awareness, { undoManager })
    ]
  });

  editor = new EditorView({ state, parent: editorDiv.value });
  editor.requestMeasure()

}

async function compile() {
  if (!editCtrl.openFile || !editCtrl.openProject) return;
  const api = new ProjectApi(editCtrl.openProject);
  const res = await api.compile(editCtrl.openFile);
  console.log(res);
  if (res.output?.document) editCtrl.lastCompiled = res.output?.document
}

onMounted(() => {
  setupEditor(editCtrl?.openFile ?? null);
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
    <button v-if="editCtrl && editCtrl.sidebarHidden.value" @click="editCtrl.toggleSidebar" class="transparent small">
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
      <button :disabled="(!editCtrl.openFile || !editCtrl.openProject)" class="right-round square small">
        <i>keyboard_arrow_down</i>
      </button>
    </nav>
  </nav>

  <div @if="editCtrl?.openFile != null" style="height: 100%; contain: size; display: block; overflow: scroll;"
    ref="editorDiv"></div>

  <p @if="editCtrl?.openFile == null" class="h5 center-align">No open file.</p>

</template>

<style scoped>
.CodeMirror {
  height: 100vh;
}
</style>

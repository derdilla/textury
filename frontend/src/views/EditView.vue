<script setup lang="ts">
import * as Y from 'yjs'
import { yCollab } from 'y-codemirror.next'
import { WebsocketProvider } from 'y-websocket'

import { EditorView, basicSetup } from "codemirror";
import { EditorState } from "@codemirror/state";
//import { javascript } from '@codemirror/lang-javascript'

import * as random from 'lib0/random'

import { useTemplateRef, onMounted } from 'vue'

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

// select a random color for this user
const userColor = usercolors[random.uint32() % usercolors.length];

const ydoc = new Y.Doc();

//const provider = new WebrtcProvider('codemirror6-demo-room', ydoc)
const wsProvider = new WebsocketProvider('ws://localhost:3000/api/v1/project/', 'test/edit', ydoc, { connect: true });

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
    //javascript(),
    yCollab(ytext, wsProvider.awareness, { undoManager })
  ]
});

const editor = useTemplateRef('editor');

onMounted(() => {
  new EditorView({ state, parent: editor.value });
});
</script>

<template>
  <div ref="editor"></div>
</template>

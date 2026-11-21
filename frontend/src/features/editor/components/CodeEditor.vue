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
import { search, searchKeymap, highlightSelectionMatches, openSearchPanel } from "@codemirror/search"
import { autocompletion, completionKeymap, closeBrackets, closeBracketsKeymap } from "@codemirror/autocomplete"
import { lintKeymap } from "@codemirror/lint"
import { latex } from 'codemirror-lang-latex';
import { typst } from 'codemirror-lang-typst';
import * as random from 'lib0/random'
import * as Y from 'yjs';

import { onMounted, ref, watch, shallowRef } from 'vue'
import { oneDark } from "@codemirror/theme-one-dark";
import { useAuthStore } from '@/shared/stores/authStore';
import { useEditorSession } from "@/features/editor/composables/editorSession";
import SearchPanel from "@/features/editor/components/SearchPanel.vue";

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
const searchOpen = ref(false);

const editor = shallowRef<EditorView | null>(null);
let isRecreatingEditor = false;

function createSearchPanel(view: EditorView) {
  searchOpen.value = true;

  return {
    dom: document.createElement("div"),
    update() {},
    destroy() {
      if (!isRecreatingEditor) {
        searchOpen.value = false;
      }
    }
  };
}

function setupEditor() {
  const wasSearchOpen = searchOpen.value;

  if (editor.value) {
    isRecreatingEditor = true;
    editor.value.destroy();
    editor.value = null;
    isRecreatingEditor = false;
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
    yCollab(yText, awareness, { undoManager }),

    // Disables editing conditionally.
    EditorView.editable.of(editorSession.canEdit.value),
    EditorState.readOnly.of(!editorSession.canEdit.value),

    search({
      createPanel: createSearchPanel
    }),

    EditorView.theme({
      ".cm-scroller": {
        fontFamily: "var(--font-mono)"
      }
    }),
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

  editor.value = new EditorView({
    state,
    parent: editorDiv.value,
  });

  if (wasSearchOpen) {
    openSearchPanel(editor.value);
  }

  editor.value.requestMeasure();
}

onMounted(() => {
  setupEditor();
});

watch(
  () => editorSession.openDocument.value,
  () => {
    setupEditor();
  }
);

watch(
  () => editorSession.canEdit.value,
  () => {
    setupEditor();
  }
);
</script>

<template>
  <div class="editor">
    <SearchPanel
      v-if="searchOpen && editor"
      :view="editor"
    />
    <div ref="editorDiv" class="editor-view"></div>
  </div>
</template>

<style scoped>
.editor {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-height: 0;
  min-width: 0;
}

.editor-view {
  flex: 1 1 0;
  min-height: 0;
  min-width: 0;
}

:deep(.cm-editor) {
  height: 100%;
  background-color: var(--surface-container);
  clip-path: inset(0px round 9px);
}

:deep(.cm-panels) {
  display: none !important;
}

:deep(.cm-gutters) {
  background-color: var(--surface-container);
  color: white;
  opacity: 0.5;
  border: none;
}
</style>

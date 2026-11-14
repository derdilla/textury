<script setup lang="ts">
import { computed } from 'vue';
import { useEditorSession } from '../composables/editorSession';
import { ConnectionState } from '../composables/editorWS';

const editorSession = useEditorSession();

const state = computed(() => editorSession.currentTree.value?.ws.state.value);
const statusText = computed(() => {
  switch (state.value) {
    case ConnectionState.Connected: {
      return 'Connected'
    }
    case ConnectionState.Disconnected: {
      return 'Disconnected'
    }
    case ConnectionState.EstablishingConnection: {
      return 'Connecting...'
    }
  };
  return 'Unknown state';
});

function onClick() {
  const ws = editorSession.currentTree.value?.ws;
  if (!ws) {
    console.warn('No websocket in editor session');
    return;
  }
  ws.connect();
}
</script>

<template>
<button class="chip no-round"
  :class="{
   'error': state !== ConnectionState.Connected
  }"
  @click="onClick"
>
  <progress v-if="state === ConnectionState.EstablishingConnection" class="small circle wavy indeterminate responsive" value="50" max="100"></progress>
  <span>{{statusText}}</span>
  <span class="tooltip right">Tap to reconnect.</span>
</button>
</template>

<style scoped>
</style>

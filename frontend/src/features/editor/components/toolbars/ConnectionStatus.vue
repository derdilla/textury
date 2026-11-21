<script setup lang="ts">
import { computed } from 'vue';
import { useEditorSession } from "@/features/editor/composables/editorSession";
import { ConnectionState } from "@/features/editor/composables/editorWS";
import FloatingElement from '@/shared/components/primitives/FloatingElement.vue';

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
  <FloatingElement ref="sidebarFloating" hover align="center">
    <template #button>
      <button class="chip no-round" @click="onClick"
        :class="{ 'error': state !== ConnectionState.Connected }">
        <progress v-if="state === ConnectionState.EstablishingConnection" class="small circle wavy indeterminate responsive" value="50" max="100"></progress>
        <span>{{statusText}}</span>
      </button>
    </template>
    <template #menu>
      <span class="tooltip bottom active">Tap to reconnect.</span>
    </template>
  </FloatingElement>
</template>

<style scoped>
</style>

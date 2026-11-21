<script setup lang="ts">
import FilePanel from "@/features/editor/components/panels/FilePanel.vue";
import OutputPanel from "@/features/editor/components/panels/OutputPanel.vue";
import ProjectPanel from "@/features/editor/components/panels/ProjectPanel.vue";
import ProjectSettingsModal from "@/features/settings/views/ProjectSettingsModal.vue";
import NotFoundView from "@/shared/views/NotFoundView.vue";
import { onMounted, onUnmounted, ref, computed } from "vue";
import { useEditorLayoutStore } from "@/features/editor/stores/editorLayoutStore";
import { useEditorSession } from "@/features/editor/composables/editorSession";

const editorLayoutStore = useEditorLayoutStore();
const editorSession = useEditorSession();

const isDragging = ref(false);
const isResizing = ref(false);
const isLeftSnapClosing = ref(false);
const isRightSnapClosing = ref(false);

const SNAP_CLOSE_THRESHOLD = 100;

let resizeTimer: ReturnType<typeof setTimeout> | null = null;
let resizeFrameId: number | null = null;

function handleWindowResize() {
  isResizing.value = true;
  editorLayoutStore.handleResize();

  if (resizeTimer) clearTimeout(resizeTimer);
  resizeTimer = setTimeout(() => {
    isResizing.value = false;
  }, 100);
}

const isProjectSettingsModalOpen = ref(false);

const isProjectNotFound = computed(() => {
  return editorSession.projectId.value && !editorSession.projectSummary.value;
});

onMounted(() => {
  editorLayoutStore.initPanelWidths();
  handleWindowResize();
  window.addEventListener("resize", handleWindowResize);
});

onUnmounted(() => {
  window.removeEventListener("resize", handleWindowResize);
  if (resizeTimer) clearTimeout(resizeTimer);
  if (resizeFrameId) cancelAnimationFrame(resizeFrameId);
});

function startResizeDrag(event: PointerEvent, isSidebar: boolean) {
  if (isSidebar ? editorLayoutStore.isLeftOverlay.value : editorLayoutStore.isRightOverlay.value) return;

  const target = event.currentTarget as HTMLElement | null;
  if (target?.setPointerCapture) {
    target.setPointerCapture(event.pointerId);
  }

  isDragging.value = true;
  const startX = event.clientX;

  const startWidth = isSidebar ? editorLayoutStore.sidebarWidth.value : editorLayoutStore.outputWidth.value;
  const startPct = isSidebar ? editorLayoutStore.sidebarPct.value : editorLayoutStore.outputPct.value;
  let currentWidth = startWidth;

  function onPointerMove(e: PointerEvent) {
    const delta = isSidebar ? e.clientX - startX : startX - e.clientX;
    currentWidth = startWidth + delta;

    if (isSidebar) {
      isLeftSnapClosing.value = currentWidth < SNAP_CLOSE_THRESHOLD;
    } else {
      isRightSnapClosing.value = currentWidth < SNAP_CLOSE_THRESHOLD;
    }

    if (resizeFrameId) cancelAnimationFrame(resizeFrameId);
    resizeFrameId = requestAnimationFrame(() => {
      if (currentWidth >= SNAP_CLOSE_THRESHOLD) {
        if (isSidebar) {
          editorLayoutStore.setSidebarWidthFromPixels(currentWidth);
        } else {
          editorLayoutStore.setOutputWidthFromPixels(currentWidth);
        }
      }
    });
  }

  function onPointerUp(e: PointerEvent) {
    if (resizeFrameId) {
      cancelAnimationFrame(resizeFrameId);
      resizeFrameId = null;
    }
    if (target?.releasePointerCapture) {
      try { target.releasePointerCapture(event.pointerId); } catch {}
    }

    isDragging.value = false;
    isLeftSnapClosing.value = false;
    isRightSnapClosing.value = false;

    window.removeEventListener("pointermove", onPointerMove);
    window.removeEventListener("pointerup", onPointerUp);

    if (currentWidth < SNAP_CLOSE_THRESHOLD) {
      if (isSidebar) {
        editorLayoutStore.closeSidebarByUser();
        editorLayoutStore.sidebarPct.value = startPct;
      } else {
        editorLayoutStore.closeOutputByUser();
        editorLayoutStore.outputPct.value = startPct;
      }
    } else {
      if (isSidebar) {
        editorLayoutStore.setSidebarWidthFromPixels(currentWidth);
      } else {
        editorLayoutStore.setOutputWidthFromPixels(currentWidth);
      }
    }
  }

  window.addEventListener("pointermove", onPointerMove);
  window.addEventListener("pointerup", onPointerUp);
}

const startDragLeft = (e: PointerEvent) => startResizeDrag(e, true);
const startDragRight = (e: PointerEvent) => startResizeDrag(e, false);
</script>

<template>
  <NotFoundView v-if="isProjectNotFound" type="project" />
  <main v-else class="responsive no-padding no-margin max">
    <!-- Backdrop -->
    <div v-if="(editorLayoutStore.isLeftOverlay.value && !editorLayoutStore.sidebarHidden.value) ||
      (editorLayoutStore.isRightOverlay.value && !editorLayoutStore.outputHidden.value)"
      class="overlay-backdrop" @click="editorLayoutStore.closeOverlays"/>

    <div class="draggable-grid no-space" :class="{ 'is-dragging': isDragging, 'is-resizing': isResizing }"
      style="height: 100vh; max-height: 100vh;">
      <!-- Project Sidebar -->
      <div class="no-padding no-margin grid-item panel-wrapper left-panel"
        :class="{
          'is-hidden': editorLayoutStore.sidebarHidden.value,
          'is-overlay': editorLayoutStore.isLeftOverlay.value,
          'will-snap-close': isLeftSnapClosing
        }"
        :style="{
          width: editorLayoutStore.sidebarHidden.value
            ? '0px'
            : editorLayoutStore.isLeftOverlay.value
              ? ''
              : `${editorLayoutStore.sidebarWidth.value}px`
        }">
        <ProjectPanel v-show="!editorLayoutStore.sidebarHidden.value"
          @openProjectSettings="isProjectSettingsModalOpen = true"
          @close="editorLayoutStore.closeSidebarByUser" class="panel active left no-margin no-padding"/>
      </div>

      <!-- Gutter Left -->
      <div
        v-if="!editorLayoutStore.sidebarHidden.value && !editorLayoutStore.isLeftOverlay.value" class="gutter-col"
        :class="{ 'will-snap-close': isLeftSnapClosing }" @pointerdown.prevent="startDragLeft">
        <div class="drag-indicator"></div>
      </div>

      <!-- Text Editor -->
      <div class="no-padding no-margin grid-item editor-item">
        <FilePanel />
      </div>

      <!-- Gutter Right -->
      <div v-if="!editorLayoutStore.outputHidden.value && !editorLayoutStore.isRightOverlay.value"
        class="gutter-col" :class="{ 'will-snap-close': isRightSnapClosing }" @pointerdown.prevent="startDragRight">
        <div class="drag-indicator"></div>
      </div>

      <!-- Output Viewer -->
      <div class="no-padding no-margin grid-item panel-wrapper right-panel"
        :class="{
          'is-hidden': editorLayoutStore.outputHidden.value,
          'is-overlay': editorLayoutStore.isRightOverlay.value,
          'will-snap-close': isRightSnapClosing
        }"
        :style="{
          width: editorLayoutStore.outputHidden.value
            ? '0px'
            : editorLayoutStore.isRightOverlay.value
              ? ''
              : `${editorLayoutStore.outputWidth.value}px`
        }">
        <OutputPanel v-show="!editorLayoutStore.outputHidden.value" @close="editorLayoutStore.closeOutputByUser"/>
      </div>
    </div>

    <!-- Project Settings Modal -->
    <ProjectSettingsModal
      v-model:open="isProjectSettingsModalOpen"
      :project-id="editorSession.projectId.value ?? ''"
    />
  </main>
</template>

<style scoped>
.draggable-grid {
  display: flex;
  height: 100vh;
  width: 100%;
  overflow: hidden;
  position: relative;
}

.draggable-grid.is-dragging {
  user-select: none;
  cursor: col-resize;
}

.draggable-grid.is-dragging .panel-wrapper,
.draggable-grid.is-resizing .panel-wrapper {
  transition: none !important;
}

.grid-item {
  min-width: 0;
  height: 100%;
  overflow: hidden;
}

.panel-wrapper {
  flex-shrink: 0;
  transition: width 0.2s cubic-bezier(0.4, 0, 0.2, 1), filter 0.15s ease;
  position: relative;
}

.panel-wrapper.will-snap-close {
  filter: brightness(0.45);
}

.editor-item {
  flex: 1;
  min-width: 0;
  height: 100%;
}

.gutter-col {
  position: relative;
  width: 15px;
  height: 100%;
  cursor: col-resize;
  background-color: var(--surface);
  flex-shrink: 0;
  user-select: none;
  z-index: 10;
}

.drag-indicator {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 5px;
  height: 80px;
  border-radius: 5px;
  background-color: var(--surface-container-highest);
  transition: background-color 0.15s ease;
}

.gutter-col.will-snap-close .drag-indicator {
  background-color: var(--primary);
}

.overlay-backdrop {
  position: fixed;
  inset: 0;
  width: 100vw;
  height: 100vh;
  z-index: 90;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: color-mix(in srgb, rgba(0, 0, 0, 0.3), var(--primary) 10%);
  padding: 1rem;
  box-sizing: border-box;
}

.dark .overlay-backdrop {
  background-color: rgba(0, 0, 0, 0.6);
}

@media (prefers-reduced-transparency: reduce) {
  .overlay-backdrop {
    background-color: color-mix(in srgb, var(--surface), #000000 10%) !important;
  }
}

.panel-wrapper.is-overlay {
  position: fixed !important;
  top: 0;
  bottom: 0;
  height: 100vh;
  z-index: 100;
}

.left-panel.is-overlay {
  left: 0;
}

.left-panel.is-overlay:not(.is-hidden) {
  width: 280px !important;
  max-width: 85vw;
}

.right-panel.is-overlay {
  right: 0;
}

.right-panel.is-overlay:not(.is-hidden) {
  width: 320px !important;
  max-width: 85vw;
}

.panel-wrapper.is-overlay.is-hidden {
  width: 0px !important;
  pointer-events: none;
}
</style>

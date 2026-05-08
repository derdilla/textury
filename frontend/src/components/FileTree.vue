<script setup lang="ts">
/* This component will need to be rewritten at some point. Here are the main functionalities
   it currently provides:
   - Hierarchical tree with collapsable directories
   - Semantic html
   - Open documents on tap
   - Menu to delete file
   - Drag files from the desktop into a directory
   - Design integrates with the rest of the app (?)

   A new component should provide:
   - Well working css - check
   - Rename/move files - check
   - Move files by dragging them across directories - check
*/

import { type FileMetadata } from '@/services/projectApi';
import { nextTick, onMounted, computed, ref } from 'vue';
import { editCtrl } from "@/services/editorController.ts";
import FloatingElement from './FloatingElement.vue';

const props = withDefaults(defineProps<{
  root: FileMetadata,
  level?: number // used for indentation
}>(), {
  level: 0
});

const children = computed(() => {
  if (props.root.file_type.Directory != null
    && props.root.file_type.Directory.children != null) {
    return props.root.file_type.Directory.children
  }
  return [];
});

const isSelected = computed(() => editCtrl?.selectedFileOrDirectory === props.root.whole_path
  && !props.root.file_type.Directory);

// Calculate the padding based on the level
const indentStyle = computed(() => {
  const basePadding = 10;
  const step = 20;
  return {
    paddingLeft: `${step * props.level + basePadding}px`,
    '--level': props.level
  };
});

const floatingElement = ref(); // Dropdown

function handleContextMenu(e: MouseEvent) {
  e.preventDefault();
  setSelected();
  floatingElement.value?.openAt(e);
}

function setSelected() {
  editCtrl.selectedFileOrDirectory = props.root.whole_path;
  console.log(`Selecting: ${props.root.whole_path}`);
  if (props.root.file_type.Text) {
    editCtrl.tree.openFile(props.root.whole_path);
  }
}

function handleDragStart(e: DragEvent) {
  if (!e.dataTransfer) return;
  // Custom key to identify as an internal move of files
  e.dataTransfer.setData('application/file-path', props.root.whole_path);
  e.dataTransfer.effectAllowed = 'move';
}

const dragCounter = ref(0);
const isDragOver = computed(() => dragCounter.value > 0);

function handleDragEnter(e: DragEvent) {
  e.preventDefault();
  dragCounter.value++;
}

function handleDragLeave() {
  dragCounter.value = Math.max(0, dragCounter.value - 1);
}

function handleDragOver(e: DragEvent) {
  e.preventDefault();
  if (!e.dataTransfer) return;

  // Controling the cursor icon
  if (e.dataTransfer.types.includes('Files')) {
    e.dataTransfer.dropEffect = 'copy';
  } else {
    e.dataTransfer.dropEffect = 'move';
  }
}

function onDrop(e: DragEvent) {
  dragCounter.value = 0;
  handleDrop(e);
}

async function handleDrop(e: DragEvent) {
  e.preventDefault();
  e.stopPropagation();
  dragCounter.value = 0; // Stop highlight effect of the drop area.

  if (!e.dataTransfer || !editCtrl.openProject) return;

  // If dropped on a file, the target directory is its parent.
  let targetDir = props.root.whole_path;
  if (!props.root.file_type.Directory) {
    targetDir = targetDir.substring(0, targetDir.lastIndexOf('/') + 1);
  }

  // SCENARIO: File/Directory Move
  const sourcePath = e.dataTransfer.getData('application/file-path');

  if (sourcePath) {
    // If target directory is the same or a subdirectory of the already existing one, abort.
    if (sourcePath === props.root.whole_path || targetDir.startsWith(sourcePath + '/')) {
      console.warn("Invalid move: Source is same as target or target is a subfolder of source.");
      return;
    }

    const fileName = sourcePath.split('/').filter(Boolean).pop();
    const destinationPath = targetDir + '/' + fileName;

    // Execute file moving.
    await handlePathChange(sourcePath, destinationPath);
    return;
  }

  // SCENARIO: File Upload
  const fileItems = [...e.dataTransfer.items]
    .map((item) => item.getAsFile())
    .filter((file) => file !== null) as File[];

  if (fileItems.length > 0) {
    for (const file of fileItems) {
      const uploadPath = targetDir + '/' + file.name;

      // TODO: share code with ProjectPanel
      let err: string | null;
      if (file.type.startsWith('text/')) {
        err = editCtrl.tree.createTextFile(uploadPath, await file.text());
      } else {
        err = editCtrl.tree.uploadAsset(uploadPath, await file.bytes());
      }
      if (err) console.error(`Upload error: ${err}`);
    }
  }
}

async function deleteThis() {
  // Close dropdown
  if (floatingElement.value) {
    floatingElement.value.isOpen = false;
  }

  if (!editCtrl.openProject) return;

  editCtrl.tree.deleteEntry(props.root.whole_path);
}

onMounted(() => {
  console.log(props.root.whole_path + props.level);
});

const isMenuOpen = ref(false);

const handleMenuToggle = (state: boolean) => {
  isMenuOpen.value = state;
};

const isRenaming = ref(false);
const editName = ref(props.root.name);
const inputRef = ref<HTMLInputElement | null>(null);

function startRename() {
  // Close dropdown
  if (floatingElement.value) {
    floatingElement.value.isOpen = false;
  }

  // Start renaming
  editName.value = props.root.name;
  isRenaming.value = true;

  nextTick(() => {
    // Focus the input.
    const el = inputRef.value;
    if (!el) return;
    el.focus();

    // Select only the part before the file extension.
    const lastDot = editName.value.lastIndexOf('.');
    if (!props.root.file_type.Directory && lastDot > 0) {
      el.setSelectionRange(0, lastDot);
    } else {
      el.select();
    }
  });
}

async function handleRename() {
  if (!isRenaming.value) return;
  const newName = editName.value.trim();

  // File name was not changed.
  if (!newName || newName === props.root.name) {
    isRenaming.value = false;
    return;
  }

  try {
    if (editCtrl.openProject) {
      const oldPath = props.root.whole_path;
      const parentPath = oldPath.substring(0, oldPath.lastIndexOf(props.root.name));
      const newPath = parentPath + newName;

      // Execute renaming on server.
      const success = await handlePathChange(oldPath, newPath);
      if (!success) {
        editName.value = props.root.name; // Reset text
      }
    }
  } catch (err) {
    console.error("Rename Error:", err);
    editName.value = props.root.name; // Reset text
  } finally {
    isRenaming.value = false;
  }
}

function cancelRename() {
  isRenaming.value = false;
  editName.value = props.root.name;
}

function handleSummaryClick(e: MouseEvent) {
  // If user is currently renaming, stop interaction.
  if (isRenaming.value) {
    e.preventDefault();
    return;
  }
  setSelected();
}

async function handlePathChange(oldPath: string, newPath: string): Promise<boolean> {
  if (!editCtrl.openProject) return false;

  const error = editCtrl.tree.moveEntry(oldPath, newPath);
  if (error) console.error(error);

  return error == null;
}

</script>

<template>

  <!--Directory-->
  <details v-if="props.root.file_type.Directory" @dragover.prevent.stop="handleDragOver" @dragenter.stop="handleDragEnter"
    @dragleave.stop="handleDragLeave" @drop.stop="onDrop">

    <!--Collapsable Header-->
    <summary draggable="true" @dragstart="handleDragStart" @click="handleSummaryClick" class="file left row"
      @contextmenu.prevent="handleContextMenu" :class="{
        'menu-active': isMenuOpen,
        'secondary-container': isSelected,
        'drag-over': isDragOver
      }" :style="indentStyle">
      <div class="icon">
        <i class="chevron-background">folder</i>
        <i class="chevron-icon">keyboard_arrow_right</i>
      </div>
      <span v-if="!isRenaming" :title="props.root.name" class="truncate-text">
        {{ props.root.name }}
      </span>
      <input v-else ref="inputRef" v-model="editName" class="rename-input" @blur="handleRename"
        @keydown.enter="handleRename" @keydown.esc="cancelRename" @click.stop.prevent @mousedown.stop @contextmenu.stop
        @dblclick.stop />
      <FloatingElement align="right" ref="floatingElement" @click.stop @toggle="handleMenuToggle">
        <template #button>
          <button tabindex="0" class="square transparent small">
            <i>more_vert</i>
          </button>
        </template>
        <template #menu>
          <menu id="dropdown" class="no-wrap no-round active surface-container shadow">
            <li @click.stop="startRename" class="no-round small left-align"><i>edit</i> Rename</li>
            <li @click.stop="deleteThis" class="no-round small left-align error-text"><i>delete</i> Delete</li>
          </menu>
        </template>
      </FloatingElement>
    </summary>

    <!--Subdirectory-->
    <div v-for="child in children" :key="child.name">
      <input type="file" id="file-input" multiple accept="*" style="display: none" />
      <FileTree :root=child :level="(level ?? 0) + 1" />
    </div>

  </details>

  <!--File-->
  <div v-else draggable="true" @dragstart="handleDragStart" @dragover.prevent.stop="handleDragOver"
    @dragenter.stop="handleDragEnter" @dragleave.stop="handleDragLeave" @drop.stop="onDrop" @click="setSelected"
    @contextmenu.prevent="handleContextMenu" class="file left row" :class="{
      'secondary-container': isSelected,
      'menu-active': isMenuOpen,
      'drag-over': isDragOver
    }" :style="indentStyle">
    <i class="icon file-icon secondary-container" v-if="props.root.file_type.Text">description</i>
    <i class="icon file-icon secondary-container" v-if="props.root.file_type.Binary">image</i>
    <span v-if="!isRenaming" :title="props.root.name" class="truncate-text">
      {{ props.root.name }}
    </span>
    <input v-else ref="inputRef" v-model="editName" class="rename-input" @blur="handleRename"
      @keydown.enter="handleRename" @keydown.esc="cancelRename" @click.stop.prevent @mousedown.stop @contextmenu.stop
      @dblclick.stop />
    <FloatingElement align="right" ref="floatingElement" @click.stop @toggle="handleMenuToggle">
      <template #button>
        <button tabindex="0" class="square transparent small">
          <i>more_vert</i>
        </button>
      </template>
      <template #menu>
        <menu id="dropdown" class="no-wrap no-round active surface-container shadow">
          <li @click.stop="startRename" class="no-round small left-align"><i>edit</i> Rename</li>
          <li @click.stop="deleteThis" class="no-round small left-align error-text"><i>delete</i> Delete</li>
        </menu>
      </template>
    </FloatingElement>
  </div>
</template>

<style scoped>
.rename-input {
  background: color-mix(in srgb, var(--inverse-surface), transparent 90%);
  border: 0px;
  color: inherit;
  font-size: inherit;
  font-family: inherit;
  padding: 0 4px;
  margin-left: 10px;
  border-radius: 4px;
  outline: none;
  width: 100%;
  flex-shrink: 1;
}

.file:has(.rename-input) {
  background-color: color-mix(in srgb, var(--inverse-surface), transparent 95%);
}

/* Highlight element when a file is dragged over it. */
details[open]>summary.drag-over,
.file.drag-over {
  background-color: color-mix(in srgb, var(--inverse-surface), transparent 85%) !important;
  outline: 2px dashed color-mix(in srgb, var(--inverse-surface), transparent 75%);
  outline-offset: -2px;
}

[draggable="true"] {
  user-select: none;
  -webkit-user-drag: element;
}

.icon {
  scale: 0.95;
}

.file-icon {
  opacity: 0.8;
  padding: 1px;
  border-radius: 25%;
}

.icon-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.chevron-background {
  z-index: 1;
  padding: 1px;
  background-color: color-mix(in srgb, var(--inverse-surface), transparent 95%);
  color: transparent;
  border-radius: 25%;
}

.chevron-icon {
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  z-index: 2;
  font-size: 26px !important;
  transition: transform 0.2s ease, opacity 0.2s ease;
  pointer-events: none;
  border-radius: 50%;
}

/* Rotate icon when directory is open. */
details[open]>summary .chevron-icon {
  transform: translate(-50%, -50%) rotate(90deg);
}

details {
  display: flex;
  align-items: stretch;
  flex-flow: column;
  width: inherit;
}

details ul {
  margin-top: 0 !important;
  margin-bottom: 0;
}

/* Making sure browsers are not ignoring interactions to the
button inside the summary which they somehow do by default. */
summary::-webkit-details-marker {
  display: none;
}

summary.file button {
  position: relative;
  z-index: 10;
  pointer-events: auto;
}

summary.file input {
  position: relative;
  z-index: 10;
  pointer-events: auto;
}

.file {
  cursor: pointer;
  padding-right: 0px;
  display: flex;
  justify-content: start;
  border-radius: 8px;
  gap: 0;
  margin-bottom: 4px;
  font-size: medium;
  position: relative;
  box-sizing: border-box;
}

/* Adding the vertical lines to indicate that a file is inside a folder. */
.file::after {
  content: '';
  position: absolute;
  top: -4px;
  bottom: 0px;
  left: 20px;
  width: calc(20px * (var(--level)));
  pointer-events: none;
  background: repeating-linear-gradient(to right,
      color-mix(in srgb, var(--inverse-surface), transparent 90%) 0px 2px,
      transparent 2px 20px);
}

.file[style*="--level: 0"]::after {
  display: none;
}

.file> :nth-child(3) {
  margin-left: auto;
}

.file> :nth-child(2) {
  margin-left: 10px;
  margin-right: 10px;
}

/* Different active indicators. */
.file:hover {
  background-color: color-mix(in srgb, var(--inverse-surface), transparent 95%);
}

.file.menu-active {
  background-color: color-mix(in srgb, var(--inverse-surface), transparent 90%);
}

/* More button is only shown when hovering or when a file is selected. */
.file button {
  opacity: 0;
  max-width: 0;
}

.file:hover button,
.file.secondary-container button,
.file.menu-active button {
  opacity: 1;
  max-width: inherit;
}

.truncate-text {
  flex-shrink: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>

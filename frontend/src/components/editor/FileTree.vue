<script setup lang="ts">
import { nextTick, computed, ref } from 'vue';
import FloatingElement from '@/components/general/FloatingElement.vue';
import { useEditorSession } from "@/composables/editorSession";
import type { FileMetadata } from '@/composables/editorFileTree';
import { useFileDrag, useFileDrop } from '@/composables/fileDragNDrop';

const editorSession = useEditorSession();

const props = defineProps<{
  root: FileMetadata,
}>();

const isSelected = computed(() => props.root.kind !== 'directory'
  && editorSession.openDocument.value?.uuid === props.root.uuid);

// Calculate the padding based on the level
const level = computed(() => {
  let level = props.root.wholePath.split('/').length - 2;
  if (props.root.kind === 'directory') level -= 1; // trailing /
  return level;
});

function styleFromLevel(level: number) {
  const basePadding = 10;
  const step = 20;
  return {
    paddingLeft: `${step * level + basePadding}px`,
    '--level': level,
  };
}

const floatingElement = ref(); // Dropdown

function handleContextMenu(e: MouseEvent) {
  e.preventDefault();
  setSelected();
  floatingElement.value?.openAt(e);
}

function setSelected() {
  if (props.root.kind === 'directory') return;
  editorSession.selectFile(props.root.uuid);
}

const { handleDragStart } = useFileDrag(props.root);
const drop = useFileDrop(props.root);

function handleDrop(e: DragEvent) {
  if (!editorSession.projectId.value) {
    drop.onDrop(e, null)
  } else {
    drop.onDrop(e, editorSession.currentTree.value)
  }
}

function closeDropdown() {
  if (floatingElement.value) {
    floatingElement.value.isOpen = false;
  }
}

async function deleteThis() {
  closeDropdown();

  if (!editorSession.projectId.value) return;

  props.root.delete();
}

const elementInCreation = ref<'text' | 'directory' | null>(null);
const fileName = defineModel<string | null>('fileName');
const fileNameRef = ref<HTMLInputElement | null>(null);
const detailsRef = ref<HTMLDetailsElement | null>(null);

function startCreating(kind: 'text' | 'directory') {
  closeDropdown();

  // Open child list
  detailsRef.value?.setAttribute('open', 'true')

  if (props.root.kind !== 'directory') return;
  if (!editorSession.projectId.value) return;

  elementInCreation.value = kind;
  switch (kind) {
    case 'text':
      fileName.value = 'New Document.tex';
      break;
    case 'directory':
      fileName.value = 'New Directory';
      break;
  }
  requestAnimationFrame(() => requestAnimationFrame(() => { // expand animation
    nextTick(() => { // edit element created
      const el = fileNameRef.value;
      if (el) focusFilenamePart(el, fileName.value || '', kind !== 'directory')
    });
  }));
}

function cancelCreation() {
  elementInCreation.value = null;
}

function submitCreation() {
  if (props.root.kind !== 'directory') return;
  if (!editorSession.projectId.value) return;
  if (!fileName.value) return;
  if (fileName.value.indexOf('/') !== -1) return;
  switch (elementInCreation.value) {
    case 'text':
      props.root.createTextFile(fileName.value);
      break;
    case 'directory':
      props.root.createDir(fileName.value);
      break;
  }
  elementInCreation.value = null;
}

const isMenuOpen = ref(false);

const handleMenuToggle = (state: boolean) => {
  isMenuOpen.value = state;
};

const isRenaming = ref(false);
const editName = ref(props.root.name);
const inputRef = ref<HTMLInputElement | null>(null);

const sortedChildren = computed(() => {
  if (props.root.kind !== 'directory') return null;
  return props.root.children.toSorted((a, b) => {
    if (a.kind === 'directory' && b.kind !== 'directory') return -1;
    if (a.kind !== 'directory' && b.kind === 'directory') return 1;
    if (a.name < b.name) return -1;
    if (a.name > b.name) return 1;
    return 0;
  });
});

function startRename() {
  closeDropdown();

  if (!props.root.rename) {
    console.info("Can't rename this type of entry");
    return;
  }

  // Start renaming
  editName.value = props.root.name;
  isRenaming.value = true;

  nextTick(() => {
    const el = inputRef.value;
    if (el) focusFilenamePart(el, editName.value, props.root.kind !== 'directory')
  });
}

function focusFilenamePart(el: HTMLInputElement, content: string, hasExtension: boolean) {
  if (!el) return;
  el.focus();

  // Select only the part before the file extension.
  const lastDot = content.lastIndexOf('.');
  if (hasExtension && lastDot > 0) {
    el.setSelectionRange(0, lastDot);
  } else {
    el.select();
  }
}

async function handleRename() {
  if (!isRenaming.value) return;
  const newName = editName.value.trim();

  // File name was not changed.
  if (!newName || newName === props.root.name) {
    isRenaming.value = false;
    return;
  }

  if (!props.root.rename) {
    console.warn("Can't rename this entry");
    return;
  }

  try {
    if (editorSession.projectId.value) {
      props.root.rename(newName);
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

</script>

<template>

  <!--Directory-->
  <details v-if="props.root.kind === 'directory'" ref="detailsRef"
    @dragover.prevent.stop="drop.onDragOver" @dragenter.stop="drop.onEnter"
    @dragleave.stop="drop.onLeave" @drop.stop="handleDrop" :class="{
      'drag-over': drop.isDragOver.value,
    }">

    <!--Collapsable Header-->
    <summary draggable="true" @dragstart="handleDragStart" @click="handleSummaryClick" class="file left row"
      @contextmenu.prevent="handleContextMenu" :class="{
        'menu-active': isMenuOpen,
        'secondary-container': isSelected,
      }" :style="styleFromLevel(level)">
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
          <menu class="dropdown no-wrap no-round active surface-container">
            <button v-if="props.root.rename" @click.stop="startRename" class="transparent no-round left-align"><i class="small">edit</i> Rename</button>
            <button @click.stop="startCreating('text')" class="transparent no-round left-align"><i class="small">note_add</i> Create File</button>
            <button @click.stop="startCreating('directory')" class="transparent no-round left-align"><i class="small">create_new_folder</i> Create Directory</button>
            <li><hr></li>
            <button @click.stop="deleteThis" class="transparent no-round left-align error-text"><i class="small">delete</i> Delete</button>
          </menu>
        </template>
      </FloatingElement>
    </summary>

    <div v-if="elementInCreation" class="file left row" :style="styleFromLevel(level + 1)">
      <i class="icon file-icon secondary-container" v-if="elementInCreation === 'text'">description</i>
      <i class="icon file-icon secondary-container" v-if="elementInCreation === 'directory'">folder</i>
      <input v-model="fileName" ref="fileNameRef" class="rename-input"
        @blur="submitCreation" @keydown.enter="submitCreation"
        @keydown.esc="cancelCreation"/>
    </div>

    <!--Subdirectory-->
    <div v-for="child in sortedChildren" :key="child.name">
      <input type="file" id="file-input" multiple accept="*" style="display: none" />
      <FileTree :root=child />
    </div>

  </details>

  <!--File-->
  <div v-else draggable="true" @dragstart="handleDragStart"
    @click="setSelected"
    @contextmenu.prevent="handleContextMenu" class="file left row" :class="{
      'secondary-container': isSelected,
      'menu-active': isMenuOpen,
    }" :style="styleFromLevel(level)">
    <i class="icon file-icon secondary-container" v-if="props.root.kind === 'text'">description</i>
    <i class="icon file-icon secondary-container" v-if="props.root.kind === 'binary'">image</i>
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
        <menu class="dropdown no-wrap no-round active surface-container shadow">
          <button v-if="props.root.rename" @click.stop="startRename" class="transparent no-round left-align"><i class="small">edit</i> Rename</button>
          <button @click.stop="deleteThis" class="transparent no-round left-align error-text"><i class="small">delete</i> Delete</button>
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
.drag-over {
  background-color: color-mix(in srgb, var(--inverse-surface), transparent 85%) !important;
  outline: 2px dashed color-mix(in srgb, var(--inverse-surface), transparent 75%);
  outline-offset: -2px;
  border-radius: 8px;
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

.dropdown hr {
  margin: 3px;
}
.dropdown i {
  margin-right: 0.5rem;
}
.dropdown {
  gap: 4px;
}
.dropdown {
  box-shadow: none !important;
}
</style>

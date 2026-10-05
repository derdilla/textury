import { computed, ref } from "vue";
import type { EditorFileTree, FileMetadata } from "./editorFileTree";

export function useFileDrag(file: FileMetadata) {
  function handleDragStart(e: DragEvent) {
    if (!e.dataTransfer) return;
    if (!file.uuid) {
      console.error(`Can't move legacy directory: ${file.wholePath}`)
      return;
    }
    // Custom key to identify as an internal move of files
    e.dataTransfer.setData('application/file-uuid', file.uuid);
    e.dataTransfer.effectAllowed = 'move';
  }

  return { handleDragStart }
}

export function useFileDrop(file: FileMetadata) {
  const dragCounter = ref<number>(0);
  const isDragOver = computed<boolean>(() => {
    return dragCounter.value > 0
  });

  function onEnter(e: DragEvent) {
    e.preventDefault();
    dragCounter.value++;
  }

  function onLeave() {
    dragCounter.value = Math.max(0, dragCounter.value - 1);
  }

  function onDragOver(e: DragEvent) {
    e.preventDefault();
    if (!e.dataTransfer) return;

    // Controling the cursor icon
    if (e.dataTransfer.types.includes('Files')) {
      e.dataTransfer.dropEffect = 'copy';
    } else {
      e.dataTransfer.dropEffect = 'move';
    }
  }

  function onDrop(e: DragEvent, tree: EditorFileTree | null) {
    dragCounter.value = 0;
    handleDrop(e, tree);
  }

  async function handleDrop(e: DragEvent, tree: EditorFileTree | null) {
    e.preventDefault();
    e.stopPropagation();
    dragCounter.value = 0; // Stop highlight effect of the drop area.

    if (!e.dataTransfer) return;

    if (!tree) {
      console.warn('Drop without open project. Doing nothing...');
      return;
    }

    // If dropped on a file, the target directory is its parent.
    let target: null | Extract<FileMetadata, { kind: 'directory' }> = null;
    if (file.kind === 'directory') {
      target = file;
    } else {
      const targetPath = file.wholePath.substring(0, file.wholePath.lastIndexOf('/') + 1);
      const found = tree.lookupEntryByPath(targetPath);
      if (found && found.kind === 'directory') target = found;
    }

    if (!target) {
      console.error(`Unable to determine directory for target "${file.wholePath}"`);
      return;
    }

    // SCENARIO: File/Directory Move
    const sourceUUID = e.dataTransfer.getData('application/file-uuid');
    if (sourceUUID) {
      const source = tree.lookupEntryByUUID(sourceUUID);
      source?.move(target);
      return;
    }

    // SCENARIO: File Upload
    for (let idx = 0; idx < e.dataTransfer.files.length; idx++) {
      const file = e.dataTransfer.files[idx];
      if (!file) continue;
      if (file.type.startsWith('text/')) {
        target.createTextFile(file.name, await file.text());
      } else {
        target.createAsset(file.name, await file.bytes());
      }
    }
  }

  return { isDragOver, onEnter, onLeave, onDragOver, onDrop };
}

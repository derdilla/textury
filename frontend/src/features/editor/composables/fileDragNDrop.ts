import { computed, ref } from "vue";
import type { EditorFileTree, FileMetadata } from "@/features/editor/composables/editorFileTree";

export function useFileDrag(file: FileMetadata) {
  function handleDragStart(e: DragEvent) {
    if (!e.dataTransfer) return;
    if (!file.uuid) {
      console.error(`Can't move legacy directory: ${file.wholePath}`)
      return;
    }
    // Custom key to identify as an internal move of files.
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

    // Controlling the cursor icon.
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

  function _isTextFile(name: string, type: string): boolean {
    if (type.startsWith('text/')) return true;
    const lower = name.toLowerCase();
    return ['.typ', '.tex', '.bib', '.cls', '.sty', '.txt', '.md', '.json', '.yaml', '.yml'].some(ext => lower.endsWith(ext));
  }

  async function _uploadFile(fileObj: File, targetDir: Extract<FileMetadata, { kind: 'directory' }>) {
    if (_isTextFile(fileObj.name, fileObj.type)) {
      targetDir.createTextFile(fileObj.name, await fileObj.text());
    } else {
      const buffer = await fileObj.arrayBuffer();
      targetDir.createAsset(fileObj.name, new Uint8Array(buffer));
    }
  }

  async function _processEntry(entry: FileSystemEntry, currentDir: Extract<FileMetadata, { kind: 'directory' }>, tree: EditorFileTree) {
    if (entry.isFile) {
      await new Promise<void>((resolve) => {
        (entry as FileSystemFileEntry).file(async (fileObj) => {
          await _uploadFile(fileObj, currentDir);
          resolve();
        });
      });
    } else if (entry.isDirectory) {
      const dirEntry = entry as FileSystemDirectoryEntry;
      currentDir.createDir(dirEntry.name);

      const nextDir = tree.lookupEntryByPath(`${currentDir.wholePath}${dirEntry.name}/`) as Extract<FileMetadata, { kind: 'directory' }> || currentDir;

      const reader = dirEntry.createReader();
      const readEntriesRecursive = async () => {
        reader.readEntries(async (entries) => {
          if (entries.length > 0) {
            for (const subEntry of entries) {
              await _processEntry(subEntry, nextDir, tree);
            }
            readEntriesRecursive();
          }
        });
      };
      readEntriesRecursive();
    }
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

    // SCENARIO: Folder & Single/Multiple File Upload
    const items = e.dataTransfer.items;
    if (items && items.length > 0) {
      const entries: FileSystemEntry[] = [];
      for (let i = 0; i < items.length; i++) {
        const entry = items[i]?.webkitGetAsEntry?.();
        if (entry) entries.push(entry);
      }

      for (const entry of entries) {
        await _processEntry(entry, target, tree);
      }
      return;
    }
  }

  return { isDragOver, onEnter, onLeave, onDragOver, onDrop };
}

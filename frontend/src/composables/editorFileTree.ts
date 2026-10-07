import type { Compiler } from "@/api/types";
import { shallowRef, type Ref, type ShallowRef } from "vue";
import * as Y from 'yjs';
import { useEditorWS, type EditorWS } from "@/composables/editorWS";
import { FilesEntry, type ProjectStateFileType } from "@/utils/crdtFilesEntry";

export const useEditorFileTree = (projectId: string): EditorFileTree => {
  const ws = useEditorWS(projectId);

  const root = shallowRef<Extract<FileMetadata, { kind: 'directory' }> | null>(null);

  /// Map from entry UUID to FileMetadata
  const entryMap: Map<string, FileMetadata> = new Map();

  _onDocUpdate();
  ws.doc.on('update', _onDocUpdate);

  function lookupEntryByPath(fullPath: string): FileMetadata | null {
    const entry = _lookupEntry({ fullPath });
    if (!entry) return null;

    const uuid = entry[0].uuid;
    if (uuid === 'corrupt') return null;

    return lookupEntryByUUID(uuid);
  }

  function lookupEntryByUUID(uuid: string): FileMetadata | null {
    return entryMap.get(uuid) || null;
  }

  /// Returns the first file where all defined arguments match.
  function _lookupEntry({ uuid, fullPath }: {
    uuid?: string,
    fullPath?: string,
  }): [FilesEntry, number] | null {
    const files = ws.doc.getArray("files");
    for (let i = 0; i < files.length; i++) {
      const file = FilesEntry.tryNew(files.get(i));
      if (file == 'corrupt') continue;
      if ((!fullPath || file.fullPath.toString() == fullPath)
        && (!uuid || file.uuid === uuid)) {
        return [file, i];
      }
    }
    return null;
  }

  function _nameValid(filename: string): boolean {
    if (filename.indexOf('/') !== -1) return false;
    if (filename === '..') return false;
    return true;
  }

  function _pathValid(path: string) {
    if (!path.startsWith('/')) return false;
    if (path.indexOf('/../') != -1) return false;
    return true;
  }

  function _createEntry<T extends ProjectStateFileType>(
    type: T,
    fullPath: string,
    args?: {
      initialData?: Uint8Array,
      initialText?: string,
    }): 'ok' | 'nameConflict' | 'invalidPath' {
    const initialData = args?.initialData;
    const initialText = args?.initialText;

    if (!_pathValid(fullPath)) return 'invalidPath';
    if (_lookupEntry({ fullPath: fullPath })) return 'nameConflict';

    const map = new Y.Map();
    const uuid: string = crypto.randomUUID();
    map.set("version", "1");
    map.set("uuid", uuid);
    map.set("type", type);
    map.set("full_path", new Y.Text(fullPath));

    if (type === 'Directory' && !fullPath.endsWith('/')) {
      return 'invalidPath';
    }
    if (type === 'File') {
      map.set("text", new Y.Text(initialText));
    }
    if (type === 'Asset') {
      if (!initialData) {
        console.error('Missing initialData argument when creating asset')
        return 'nameConflict';
      }
      ws.send({
        t: 'setAsset',
        uuid: uuid,
        data: initialData.toBase64(),
      });
    }

    const files = ws.doc.getArray("files");
    files.push([map]);
    return 'ok';
  }

  function _computeTree():  Extract<FileMetadata, { kind: 'directory' }> | 'corrupt' {
    const files = ws.doc.getArray("files");
    const base: Extract<FileMetadata, { kind: 'directory' }> = {
      kind: 'directory',
      wholePath: '/',
      name: '', rename: null,
      children: [],
      delete: () => {
        for (const f of base.children) f.delete();
      },
      move: (newParent) => {
        for (const f of base.children) f.move(newParent);
      },
      createDir: (filename: string): 'ok' | 'nameConflict' | 'invalidPath' => {
        if (!_nameValid(filename)) return 'invalidPath';
        return _createEntry('Directory', `/${filename}/`)
      },
      createTextFile: (filename: string, initialText?: string): 'ok' | 'nameConflict' | 'invalidPath' => {
        if (!_nameValid(filename)) return 'invalidPath';
        return _createEntry('File', `/${filename}`, { initialText });
      },
      createAsset: (filename: string, content: Uint8Array): 'ok' | 'nameConflict' | 'invalidPath' => {
        if (!_nameValid(filename)) return 'invalidPath';
        return _createEntry('Asset', `/${filename}`, { initialData: content });
      },
    };

    entryMap.clear();
    for (const f of files) {
      const entry = FilesEntry.tryNew(f);
      if (entry === 'corrupt') continue;
      const filePathObj = entry.fullPath;
      if (filePathObj === 'corrupt') continue;
      const filePath = filePathObj.toString();
      if (!filePath.startsWith("/")) {
        console.warn(`Malformed path "${filePath}" doesn't start with "/"`);
        continue;
      }
      if (entry.type === 'corrupt') continue;
      if (entry.uuid === 'corrupt') continue;

      // Ensure every directory in the path exists. May invent directories in case of corrupt files.
      let curr: Extract<FileMetadata, { kind: 'directory' }> = base;

      let directoryPathSegments = filePath.split("/");
      if (entry.type !== 'Directory') {
        directoryPathSegments = directoryPathSegments.slice(0, -1);
      }
      for (const pathComponent of directoryPathSegments) {
        if (pathComponent === "") continue;

        // Find existing directory with this name
        let found = false;
        for (const child of curr.children) {
          if (child.name == pathComponent) {
            if (child.kind !== 'directory') {
              console.warn(`File tree entry "${filePath}" is potentially child of non-directory entry ${child.wholePath}`);
              continue; // proceeds to imagine a directory with identical name...
            }
            curr = child;
            found = true;
            break;
          }
        }

        if (!curr.wholePath.endsWith('/')) {
          console.warn(`Malformed path "${curr.wholePath}" doesn't end with "/"`)
          curr.wholePath += '/';
        }

        // Create new directory
        if (!found) {
          const parentPath = curr.wholePath;
          const dirName = pathComponent;
          const wholePath = `${parentPath}${dirName}/`;

          const children: FileMetadata[] = [];
          const newDirectory: Extract<FileMetadata, { kind: 'directory' }> = {
            wholePath,
            name: dirName,
            kind: 'directory',
            children,
            // In case there is a backing directory object in the files array, these get extended.
            rename: (newName: string) => {
              if (!_nameValid(newName)) return;

              newDirectory.name = newName;
              newDirectory.wholePath = `${parentPath}${newName}/`
              for (const c of children) c.move(newDirectory);
            },
            delete: () => {
              for (const c of children) c.delete();
            },
            move: (newParent) => {
              if (newParent.wholePath.startsWith(newDirectory.wholePath)) {
                console.warn("Can't move to child directory");
                return;
              }
              newDirectory.wholePath = `${newParent.wholePath}${dirName}/`;
              for (const c of children) c.move(newDirectory);
            },
            createDir: (filename: string): 'ok' | 'nameConflict' | 'invalidPath' => {
              if (!_nameValid(filename)) return 'invalidPath';
              return _createEntry('Directory', `${wholePath}${filename}/`)
            },
            createTextFile: (filename: string, initialText?: string): 'ok' | 'nameConflict' | 'invalidPath' => {
              if (!_nameValid(filename)) return 'invalidPath';
              return _createEntry('File', `${wholePath}${filename}`, { initialText });
            },
            createAsset: (filename: string, content: Uint8Array): 'ok' | 'nameConflict' | 'invalidPath' => {
              if (!_nameValid(filename)) return 'invalidPath';
              return _createEntry('Asset', `${wholePath}${filename}`, { initialData: content });
            },
          };

          curr.children.push(newDirectory);
          curr = newDirectory;
        }
      }

      const renameFn = (newName: string) => {
        const pathYText = entry.fullPath;
        if (pathYText === 'corrupt') return;
        const pathString = pathYText.toString();

        const fileNameStart = pathString.lastIndexOf('/') + 1;

        pathYText.delete(fileNameStart, pathString.length - fileNameStart);
        pathYText.insert(fileNameStart, newName);
      };
      const moveFn = (newParent: Extract<FileMetadata, { kind: 'directory' }>) => {
        const pathYText = entry.fullPath;
        if (pathYText === 'corrupt') return;
        const pathString = pathYText.toString();

        let fileNameStart;
        if (pathString.endsWith('/')) {
          // We don't want to delete the name of directories
          fileNameStart = pathString.lastIndexOf('/', pathString.length - 2) + 1;
        } else {
          fileNameStart = pathString.lastIndexOf('/') + 1;
        }

        ws.doc.transact(tr => {
          pathYText.delete(0, fileNameStart);
          pathYText.insert(0, newParent.wholePath);
        });
      };
      const deleteFn = () => {
        const idx = _lookupEntry({ uuid: entry.uuid })?.[1];
        if (idx !== 0 && idx === undefined) {
          console.warn("Didn't find file to delete");
          return;
        }
        files.delete(idx);
      };

      const baseEntry = {
        uuid: entry.uuid,
        wholePath: filePath,
        name: filePath.substring(filePath.lastIndexOf('/') + 1),
        rename: renameFn,
        move: moveFn,
        delete: deleteFn,
      };

      // Link files entry to tree object
      switch (entry.type) {
        case 'Asset':
          const asset: FileMetadata = {
            kind: 'binary',
            ...baseEntry,
            update: (newContent) => {
              ws.send({
                t: 'setAsset',
                uuid: entry.uuid,
                data: newContent.toBase64(),
              });
            }
          }
          entryMap.set(entry.uuid, asset);
          curr.children.push(asset);
          break;
        case 'File':
          const document = entry.text;
          if (document === 'corrupt') continue;
          if (!document) {
            console.error(`Tree corruption: Text file "${filePath}" doesn't contain text property`);
            continue;
          }
          const file: FileMetadata = {
            kind: 'text',
            ...baseEntry,
            document,
          }
          entryMap.set(entry.uuid, file);
          curr.children.push(file);
          break;
        case 'Directory':
          // We need to modify the directory created in the loop, to maintain references.
          // Maybe files without directory come before files with.
          const oldRenameFn = curr.rename;
          const oldMoveFn = curr.move;
          const oldDeleteFn = curr.delete;
          curr.uuid = entry.uuid;
          curr.rename = (newName) => {
            if (!_nameValid(newName)) return;
            if (oldRenameFn) oldRenameFn(newName);
            renameFn(newName);
          }
          curr.move = (newParent) => {
            moveFn(newParent);
            oldMoveFn(newParent);
          }
          curr.delete = () => {
            oldDeleteFn();
            deleteFn();
          }
          entryMap.set(entry.uuid, curr);
          break;
      }
    }

    return base;
  }

  function _onDocUpdate() {
    const result = _computeTree();
    if (result === 'corrupt') {
      console.error("Didn't update tree");
      return;
    }
    root.value = result;
  }

  return {
    root,
    lookupEntryByPath,
    lookupEntryByUUID,
    ws,
  }
}

export type EditorFileTree = {
  root: ShallowRef<Extract<FileMetadata, { kind: 'directory' }> | null>,
  lookupEntryByPath: (fullPath: string) => FileMetadata | null,
  lookupEntryByUUID: (uuid: string) => FileMetadata | null,
  ws: EditorWS,
}

export type FileMetadata = {
  wholePath: string,
  name: string,
  rename: null | ((newName: string) => void),
  move: (newParent: Extract<FileMetadata, { kind: 'directory' }>) => void,
  delete: () => void,
} & FileType;

type FileType = {
  kind: 'text',
  uuid: string,
  document: Y.Text,
} | {
  kind: 'binary',
  uuid: string,
  update: (newContent: Uint8Array) => void,
} | {
  kind: 'directory',
  uuid?: string,
  children: FileMetadata[],
  createDir: (filename: string) => 'ok' | 'nameConflict' | 'invalidPath',
  createTextFile: (filename: string, initialText?: string) => 'ok' | 'nameConflict' | 'invalidPath',
  createAsset: (filename: string, content: Uint8Array) => 'ok' | 'nameConflict' | 'invalidPath',
};

interface EditorDocument {
  readonly uuid: string,
  readonly content: Y.Text,
  log: Ref<null | string>,
  pdf: Ref<null | string>,
  startCompile: (compiler: Compiler) => void,
}

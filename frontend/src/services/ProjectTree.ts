import {type FileMetadata} from "@/services/api/projectApi";
import {CustomWSProvider} from "@/services/CustomWSProvider.ts";
import * as Y from "yjs";
import { Text as YText, type Doc as YDoc, type Map as YMap, type Array as YArray } from "yjs";
import {editCtrl} from "@/services/editorController.ts";
import {markRaw} from "vue";
import {WebrtcProvider} from "y-webrtc";

export const EnableWebRTC: boolean = false;

type ProjectStateFileType = "File" | "Directory" | "Asset";

export class ProjectTree {
  private doc: YDoc | null = null;
  dirIndex: FileMetadata = {
    whole_path: "/",
    name: "",
    file_type: {
      "Directory": {
        children: [],
      }
    }
  };
  openDocument: YText | null = null;
  openDocumentPath: string | null = null;
  openDocumentUUID: string | null = null;
  wsProvider: CustomWSProvider | null = null;

  updateProject(newDoc: YDoc, wsApi: string) {
    // TODO: cancel old observer
    this.doc = markRaw(newDoc);

    if (!this.doc) return;
    const array = this.doc.getArray("files");
    this.dirIndex = _computeTree(array);

    this.wsProvider?.dispose();
    this.rtcProvider?.destroy();
    this.wsProvider = markRaw(new CustomWSProvider(this.doc, wsApi,
      (uuid, log) => {
        if (uuid !== this.openDocumentUUID) return;
        editCtrl.log = log;
      },
      (uuid, pdf) => {
        console.log(pdf);
        if (uuid !== this.openDocumentUUID) return;
        editCtrl.lastCompiled = pdf ?? '';
      },
    ));

    array?.observeDeep(allEvents => {
      /* TODO: reenable change filtering (#3)
      console.debug(`observeDeep called with ${allEvents.length} events`);
      console.debug(allEvents);
      let treeChanged = false;
      for (const event of allEvents) {
        if (event.target == array) {
          // Element got added or deleted
          treeChanged = true;
          break;
        }

        if (event.target instanceof Y.Text) {
          const parent = event.target.parent;
          if (parent instanceof Y.Map) {
            if (parent.get("full_path") === event.target) {
              // Element got moved
              treeChanged = true;
              break;
            }
          }
        }
      }

      if (!treeChanged) {
        console.debug("Tree didn't change")
        return;
      }*/

      this.dirIndex = _computeTree(array);
    })
  }

  createDir(full_path: string): string | null {
    const err = _validateCreation(this.doc, full_path);
    if (err != null) return err;

    const [_, map] = _createEntry(full_path, "Directory");
    this.doc!.getArray("files")!.push([map]); // existence validated in _validateCreation

    return null;
  }

  createTextFile(full_path: string, initialText?: string): string | null {
    console.log('createTextFile()', full_path, initialText);
    const err = _validateCreation(this.doc, full_path);
    if (err != null) return err;

    const [_, map] = _createEntry(full_path, "File");
    console.debug("map created")
    map.set("text", new Y.Text(initialText));
    this.doc!.getArray("files")!.push([map]); // existence validated in _validateCreation

    return null;
  }

  uploadAsset(full_path: string, data: Uint8Array): string | null {
    const err = _validateCreation(this.doc, full_path);
    if (err != null) return err;

    const [uuid, map] = _createEntry(full_path, "Asset");
    this.wsProvider?.setAsset(uuid, data);

    this.doc!.getArray("files")!.push([map]); // existence validated in _validateCreation

    return null;
  }

  moveEntry(old_path: string, new_path: string): string | null {
    if (old_path === new_path) return null;
    const err = _validateCreation(this.doc, new_path);
    if (err != null) return err;

    for (const file of this.doc!.getArray("files")) {
      if (!(file instanceof Y.Map)) continue;

      const filePath = file.get("full_path");
      if (filePath instanceof Y.Text && filePath.toString() === old_path) {
        this.doc?.transact(() => {
          // TODO: stop using a yText for this since there is no benefit; needs server updates
          filePath.delete(0, filePath.length)
          filePath.insert(0, new_path);
        });
        if (old_path == this.openDocumentPath) {
          this.openDocumentPath = new_path;
        }
        return null;
      }
    }

    return null;
  }

  deleteEntry(full_path: string){
    console.info(`deleteEntry(${full_path})`);
    if (!this.doc) return;

    const array = this.doc.getArray("files");

    for (let i = 0; i < array.length; i++) {
      const file = array.get(i);

      if (!(file instanceof Y.Map)) continue;

      const filePath = file.get("full_path");
      console.debug(` - ${filePath.toString()}`);
      if (filePath == full_path) {
        if (filePath == this.openDocumentPath) {
          this.openDocumentPath = null;
          this.openDocumentUUID = null;
          this.openDocument = null;
        }
        array.delete(i);
        return;
      }
    }
    console.warn("Unable to delete file: not found");
  }

  openFile(full_path: string): void {
    console.info(`openFile(${full_path})`);
    if (!this.doc) return;

    for (const file of this.doc.getArray("files")) {
      if (!(file instanceof Y.Map)) continue;

      const filePath = file.get("full_path");
      if (filePath instanceof Y.Text && filePath.toString() === full_path) {
        const type = file.get("type") as ProjectStateFileType | null;
        if (type !== "File") {
          console.warn(`Unrecognized type: ${type}`);
          return;
        }

        const text = file.get("text");
        if (!(text instanceof Y.Text)) return;

        console.debug(`Updating open document to ${text}`);
        this.openDocument = text;
        this.openDocumentPath = full_path;
        this.openDocumentUUID = file.get("uuid");
        return;
      }
    }

    console.info("Unable to find a file");
  }

}

function _validateCreation(doc: YDoc | null, path: string): string | null {
  if (doc == null) return "Document is not loaded";
  if (!path.startsWith('/')) return "Path must start with /";
  for (const file of doc.getArray("files")) {
    if (!(file instanceof Y.Map)) {
      console.warn(`Unrecognized file entity: ${file}`);
      continue;
    }
    const exisingPath = file.get("full_path");
    if (!(exisingPath instanceof Y.Text)) {
      console.warn(`Malformed path of file ${file}: ${exisingPath}`);
      continue;
    }
    if (path === exisingPath.toString()) return "Path already exists";
  }
  return null;
}

function _createEntry(path: string, type: ProjectStateFileType): [string, Y.Map<unknown>] {
  const map = new Y.Map();
  const uuid: string = crypto.randomUUID();
  map.set("version", "1");
  map.set("uuid", uuid);
  map.set("type", type);
  map.set("full_path", new Y.Text(path));
  return [uuid, map];
}

function _computeTree(files: YArray<unknown>): FileMetadata {
  console.log(`_computeTree() for ${files.length} files`);
  const base: FileMetadata = {
    whole_path: "/",
    name: "",
    file_type: {
      "Directory": {
        children: [],
      }
    }
  };
  for (const f of files) {
    //console.log(`Item: ${f}`);
    if (!(f instanceof Y.Map)) continue;
    const filePathObj = f.get("full_path");
    //console.log(`- ${filePathObj}`);
    if (!(filePathObj instanceof Y.Text)) continue;
    const filePath = filePathObj.toString();
    //console.log(`  (${filePath})`);
    if (!filePath.startsWith("/")) {
      console.warn("Malformed path");
      continue;
    }

    let curr = base;
    for (const comp of filePath.split("/")) {
      if (comp === "") continue;
      let found = false;
      for (const child of curr.file_type.Directory?.children ?? []) {
        if (child.name == comp) {
          curr = child;
          found = true;
          break;
        }
      }

      if (!found) {
        const newElement = {
          whole_path: `${curr.whole_path}${comp}/`,
          name: comp,
          file_type: {
            "Directory": {
              children: [],
            }
          }
        };
        curr.file_type.Directory?.children.push(newElement);
        curr = newElement;
      }
    }

    // Correct last elements type
    const type: ProjectStateFileType = f.get("type");
    // TODO: get unknown values from somewhere
    switch (type) {
      case "Directory":
        // Done in loop
        break;
      case "Asset":
        curr.file_type = {
          Binary: {
            size: 1,
          }
        };
        curr.whole_path = filePath;
        break;
      case "File":
        curr.file_type = {
          Text: {
            size: 1,
          }
        };
        curr.whole_path = filePath;
        break;
    }

  }
  return base;
}

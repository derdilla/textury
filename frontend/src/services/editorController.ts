import { reactive, watch } from "vue"
import { ProjectApi, type ProjectIndex } from "./projectApi"
import type { PrivateAccountInfo } from "./managementApi";
import * as Y from "yjs";
import {ProjectTree} from "@/services/ProjectTree.ts";

export interface EditController {
  user: null | PrivateAccountInfo,
  sidebarHidden: boolean,
  outputHidden: boolean,
  openProject: string | null,
  selectedFileOrDirectory: string | null,
  lastCompiled: string,
  compilerPref: null | Compiler,
  logType: 'ok' | 'warn' | 'err',
  showLog: boolean,
  log: string | null,
  index: ProjectIndex | null,
  tree: ProjectTree,
}

export const editCtrl: EditController = reactive({
  user: null,
  sidebarHidden: false,
  outputHidden: false,
  openProject: null,
  selectedFileOrDirectory: null,
  lastCompiled: "/sampleDoc.pdf",
  compilerPref: null,
  logType: 'ok',
  showLog: false,
  log: null,
  index: null,
  tree: new ProjectTree(),
});

export type Compiler = "Typst" | "PdfLatex" | "XeLatex" | "LuaLatex";

async function updateIndex(api: ProjectApi) {
  let index = await api.index();
  if (!index) index = await api.index();
  if (index) {
    editCtrl.index = index;
  }
}

watch(
  () => editCtrl.openProject,
  async (newProject) => {
    if (!newProject) return;
    const api = new ProjectApi(newProject);
    await updateIndex(api);

    editCtrl.tree.updateProject(new Y.Doc(), api.ySocketUrl())
  },
);

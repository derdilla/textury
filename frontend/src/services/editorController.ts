import { computed, type ComputedRef, reactive, type Reactive, watch } from "vue"
import { ProjectApi, type FileMetadata, type ProjectIndex } from "./projectApi"
import type { PrivateAccountInfo } from "./managementApi";

export interface EditController {
  user: null | PrivateAccountInfo,
  sidebarHidden: boolean,
  outputHidden: boolean,
  openProject: string | null,
  /// Whole path of the currently open document
  openFile: string | null,
  lastCompiled: string,
  compilerPref: null | Compiler,
  logType: 'ok' | 'warn' | 'err',
  showLog: boolean,
  log: string | null,
  dirIndex: FileMetadata | null,
  index: ProjectIndex | null,
}

export const editCtrl: EditController = reactive({
  user: null,
  sidebarHidden: false,
  outputHidden: false,
  openProject: null,
  openFile: null,
  lastCompiled: "/sampleDoc.pdf",
  compilerPref: null,
  logType: 'ok',
  showLog: false,
  log: null,
  dirIndex: null,
  index: null,
});

export enum Compiler {
  typst,
  pdflatex,
  xelatex,
  lualatex,
}

export async function updateIndex(api: ProjectApi) {
  const index = await api?.index();
  if (index) {
    editCtrl.index = index;
    editCtrl.dirIndex = index.root;
  }
}

watch(
  () => editCtrl.openProject,
  async (newProject) => {
    if (!newProject) return;
    await updateIndex(new ProjectApi(newProject));
  },
);

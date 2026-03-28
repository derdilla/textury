import { computed, type ComputedRef, reactive, type Reactive } from "vue"
import type { ProjectApi } from "./projectApi"

export interface EditController {
  sidebarHidden: boolean,
  openProject: string | null,
  /// Whole path of the currently open document
  openFile: string | null,
  lastCompiled: string,
  compilerPref: null | Compiler,
  logType: 'ok' | 'warn' | 'err',
  showLog: boolean,
  log: string | null,
}

export const editCtrl: EditController = reactive({
  sidebarHidden: false,
  openProject: null,
  openFile: null,
  lastCompiled: "/sampleDoc.pdf",
  compilerPref: null,
  logType: 'ok',
  showLog: false,
  log: null,
});

export enum Compiler {
  typst,
  pdflatex,
  xelatex,
  lualatex,
}

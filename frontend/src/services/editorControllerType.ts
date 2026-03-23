import type { ComputedRef, Ref } from "vue"
import type { ProjectApi } from "./projectApi"

export type EditorController = {
  sidebarHidden: Ref<boolean>,
  toggleSidebar: (_: any) => void,
  openProject: Ref<string>,
  api: ComputedRef<ProjectApi>,
  /// Whole path of the currently open document
  openFile: Ref<string | null>,
}

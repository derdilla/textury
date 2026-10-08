import { useStorage } from '@vueuse/core'

export const useEditorLayoutStore = () => {
  const sidebarHidden = useStorage('editor_sidebar_hidden', false)
  const outputHidden = useStorage('editor_output_hidden', false)
  const showLog = useStorage('editor_show_log', false)

  function toggleSidebar() {
    sidebarHidden.value = !sidebarHidden.value
  }

  function toggleOutput() {
    outputHidden.value = !outputHidden.value
  }

  function toggleLog() {
    showLog.value = !showLog.value
  }

  return {
    sidebarHidden,
    outputHidden,
    showLog,
    toggleSidebar,
    toggleOutput,
    toggleLog,
  }
}

import { useStorage } from '@vueuse/core'

export type ProjectViewOption = 'grid' | 'list'

export const useProjectsLayoutStore = () => {
  const view = useStorage<ProjectViewOption>('projects_view', 'grid')

  function setView(mode: ProjectViewOption) {
    view.value = mode
  }

  function toggleView() {
    view.value = view.value === 'grid' ? 'list' : 'grid'
  }

  return {
    view,
    setView,
    toggleView,
  }
}

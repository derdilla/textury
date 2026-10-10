import { useStorage } from '@vueuse/core'

export type ProjectViewOption = 'grid' | 'list'

export type ProjectTemplateOption = 'none' | 'latex' | 'typst'

export const useProjectsLayoutStore = () => {
  const view = useStorage<ProjectViewOption>('projects_view', 'grid')
  const lastUsedTemplate = useStorage<ProjectTemplateOption>('projects_template', 'latex')

  function setView(mode: ProjectViewOption) {
    view.value = mode
  }

  function toggleView() {
    view.value = view.value === 'grid' ? 'list' : 'grid'
  }

  function getLastUsedTemplate() {
    return lastUsedTemplate.value
  }

  function setLastUsedTemplate(template: ProjectTemplateOption) {
    lastUsedTemplate.value = template
  }

  return {
    view,
    setView,
    toggleView,
    getLastUsedTemplate,
    setLastUsedTemplate,
  }
}

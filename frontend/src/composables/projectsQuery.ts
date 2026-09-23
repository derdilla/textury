import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useProjectsStore } from '@/stores/projects'
import type { ProjectSummary } from '@/api/types'
import { parseCustomDate } from '@/utils/formatters'

export type ProjectSortOption = 'updatedAt' | 'createdAt' | 'title'

function sortProjects(projects: ProjectSummary[], sortBy: ProjectSortOption): ProjectSummary[] {
  return [...projects].sort((a, b) => {
    if (sortBy === 'title') {
      return (a.title || '').localeCompare(b.title || '', undefined, { sensitivity: 'base' });
    }
    const dateA = parseCustomDate(sortBy === 'createdAt' ? a.createdAt : a.updatedAt);
    const dateB = parseCustomDate(sortBy === 'createdAt' ? b.createdAt : b.updatedAt);
    return dateB - dateA;
  });
}

export const useProjectsQuery = () => {
  const route = useRoute()
  const router = useRouter()
  const projectsStore = useProjectsStore()

  // Update query parameters in url.
  function updateQuery(newParams: Record<string, string | undefined>) {
    const query = { ...route.query, ...newParams }
    Object.keys(query).forEach((key) => {
      if (query[key] === undefined || query[key] === '') {
        delete query[key]
      }
    })
    router.replace({ query })
  }

  // SEARCHING

  const searchQuery = computed(() => (route.query.search as string) || '')
  const hasSearchQuery = computed(() => searchQuery.value.trim().length > 0)

  function setSearchQuery(query: string) {
    updateQuery({ search: query.trim() || undefined })
  }

  function clearSearchQuery() {
    updateQuery({ search: undefined })
  }

  // TAG FILTERING

  const selectedTagFilters = computed<string[]>(() => {
    const raw = route.query.tags
    if (typeof raw === 'string' && raw.trim()) {
      return raw.split(',').map((t) => t.trim()).filter(Boolean)
    }
    if (Array.isArray(raw)) {
      return raw.filter((t): t is string => typeof t === 'string')
    }
    return []
  })

  const hasSelectedTagFilters = computed(() => selectedTagFilters.value.length > 0)

  // Determines if tags exist in the workspace to keep layout structure consistent regardless of search filter results.
  const hasAnyTags = computed(() =>
    selectedTagFilters.value.length > 0 ||
    projectsStore.projects.value.some((p) => (p.metadata?.tags?.length ?? 0) > 0)
  )

  // Calculate available tags from projects matching selected tags and the search query.
  // Incompatible tags (tags not present on any project matching all filters) are hidden.
  // Selected tags are always retained so users can deselect them.
  // All tags are sorted alphabetically.
  const availableTags = computed(() => {
    const query = searchQuery.value.toLowerCase().trim()
    // Find projects that match both all currently selected tags and the search query.
    const matchingProjects = projectsStore.projects.value.filter((p) => {
      const currentTags = p.metadata?.tags || []
      const matchesTags = selectedTagFilters.value.every((tag) => currentTags.includes(tag))
      const matchesQuery = !query || p.title?.toLowerCase().includes(query)
      return matchesTags && matchesQuery
    })

    // Ensure selected tags are present so users can deselect them.
    const tagsSet = new Set<string>(selectedTagFilters.value)
    // Collect tags from projects that match selected tags and search query.
    matchingProjects.forEach((p) => p.metadata?.tags?.forEach((tag) => tagsSet.add(tag)))

    return Array.from(tagsSet).sort((a, b) => a.localeCompare(b, undefined, { sensitivity: 'base' }))
  })

  function setSelectedTagFilters(tags: string[]) {
    updateQuery({ tags: tags.length ? tags.join(',') : undefined })
  }

  function toggleTagFilter(tag: string) {
    const current = [...selectedTagFilters.value]
    const index = current.indexOf(tag)
    if (index > -1) {
      current.splice(index, 1)
    } else {
      current.push(tag)
    }
    setSelectedTagFilters(current)
  }

  function clearSelectedTagFilters() {
    updateQuery({ tags: undefined })
  }

  // SORTING

  const sortBy = computed<ProjectSortOption>(() => {
    const raw = route.query.sort as ProjectSortOption
    if (['updatedAt', 'createdAt', 'title'].includes(raw)) {
      return raw
    }
    return 'updatedAt'
  })

  function setSortBy(sort: ProjectSortOption) {
    updateQuery({ sort: sort === 'updatedAt' ? undefined : sort })
  }

  // OUTPUT

  // Filter and sort projects based on selected tags, search query, and sorting criteria.
  const filteredProjects = computed(() => {
    const query = searchQuery.value.toLowerCase().trim()
    const filtered = projectsStore.projects.value.filter((project) => {
      const currentTags = project.metadata?.tags || []
      const matchesTags = selectedTagFilters.value.length === 0 || selectedTagFilters.value.every((t) => currentTags.includes(t))
      const matchesQuery = !query || project.title?.toLowerCase().includes(query)
      return matchesTags && matchesQuery
    })

    return sortProjects(filtered, sortBy.value)
  })

  function clearAllFilters() {
    updateQuery({ search: undefined, tags: undefined })
  }

  return {
    // Search Query
    searchQuery,
    hasSearchQuery,
    setSearchQuery,
    clearSearchQuery,

    // Tag Filters
    selectedTagFilters,
    hasSelectedTagFilters,
    hasAnyTags,
    availableTags,
    setSelectedTagFilters,
    toggleTagFilter,
    clearSelectedTagFilters,

    // Sorting
    sortBy,
    setSortBy,

    // Result
    filteredProjects,

    clearAllFilters,
  }
}
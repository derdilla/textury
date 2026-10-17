import { useStorage } from '@vueuse/core'

export const useSidebarLayoutStore = (pageId: string) => {
  const sidebarHidden = useStorage(`${pageId}_sidebar_hidden`, false)
  const sidebarClosedByForce = useStorage(`${pageId}_sidebar_closed_by_force`, false)

  function closeSidebarByForce() {
    sidebarHidden.value = true
    sidebarClosedByForce.value = true
  }

  function closeSidebar() {
    // When sidebar was closed by force then keep force but otherwise release (= unchanged).
    sidebarHidden.value = true
  }

  function closeSidebarByUser() {
    sidebarHidden.value = true
    sidebarClosedByForce.value = false
  }

  function openSidebar() {
    sidebarHidden.value = false
  }

  function toggleSidebarByUser() {
    if (sidebarHidden.value) {
      openSidebar();
    }
    else {
      closeSidebarByUser();
    }
  }

  function thresholdReached(newValue: boolean) {
    if (newValue) {
      if (!sidebarHidden.value) {
        closeSidebarByForce();
      }
    } else {
      if (sidebarClosedByForce.value) {
        openSidebar();
        sidebarClosedByForce.value = false;
      }
    }
  }

  return {
    sidebarHidden,
    sidebarClosedByForce,
    closeSidebarByForce,
    closeSidebar,
    closeSidebarByUser,
    openSidebar,
    toggleSidebarByUser,
    thresholdReached,
  }
}

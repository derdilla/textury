import { ref, computed, watch } from 'vue'
import { useStorage } from '@vueuse/core'

export const MIN_SIDEBAR_PX = 200
export const MIN_OUTPUT_PX = 300
export const MIN_EDITOR_PX = 400

export const MIN_THREE_PANEL_PX = MIN_SIDEBAR_PX + MIN_EDITOR_PX + MIN_OUTPUT_PX
export const MIN_TWO_PANEL_PX = MIN_EDITOR_PX + MIN_OUTPUT_PX

const sidebarHidden = useStorage('editor_sidebar_hidden', false)
const outputHidden = useStorage('editor_output_hidden', false)
const showLog = useStorage('editor_show_log', false)

const sidebarClosedByForce = useStorage('editor_sidebar_closed_by_force', false)
const outputClosedByForce = useStorage('editor_output_closed_by_force', false)

const savedScreenWidth = useStorage('editor_screen_width', 0)

const sidebarPct = useStorage('editor_sidebar_pct', 20)
const outputPct = useStorage('editor_output_pct', 40)

const DEFAULT_SIDEBAR_PCT = 20
const DEFAULT_OUTPUT_PCT = 40

const windowWidth = ref(typeof window !== 'undefined' ? window.innerWidth : 1200)

const isLeftOverlay = computed(() => windowWidth.value < MIN_THREE_PANEL_PX)
const isRightOverlay = computed(() => windowWidth.value < MIN_TWO_PANEL_PX)

const layout = computed(() => {
  const width = windowWidth.value
  const sidebarHiddenState = sidebarHidden.value
  const outputHiddenState = outputHidden.value
  const sidebarPercentage = sidebarPct.value
  const outputPercentage = outputPct.value
  const editorPercentage = Math.max(0, 100 - sidebarPercentage - outputPercentage)

  if (width <= 0) return { sidebarPx: 0, outputPx: 0 }

  let sidebarPx = 0
  let outputPx = 0

  if (!isLeftOverlay.value && !isRightOverlay.value) {
    if (!sidebarHiddenState && !outputHiddenState) {
      sidebarPx = Math.max(MIN_SIDEBAR_PX, Math.round((sidebarPercentage / 100) * width))
      outputPx = Math.max(MIN_OUTPUT_PX, Math.round((outputPercentage / 100) * width))

      const overflow = (sidebarPx + outputPx) - (width - MIN_EDITOR_PX)
      if (overflow > 0) {
        const sidebarSlack = Math.max(0, sidebarPx - MIN_SIDEBAR_PX)
        const outputSlack = Math.max(0, outputPx - MIN_OUTPUT_PX)
        const totalSlack = sidebarSlack + outputSlack

        if (totalSlack > 0 && overflow <= totalSlack) {
          const sidebarReduce = Math.round(overflow * (sidebarSlack / totalSlack))
          sidebarPx -= sidebarReduce
          outputPx -= (overflow - sidebarReduce)
        } else {
          sidebarPx = MIN_SIDEBAR_PX
          outputPx = MIN_OUTPUT_PX
        }
      }
    } else if (!sidebarHiddenState && outputHiddenState) {
      const visiblePercentage = sidebarPercentage + editorPercentage
      const rawSidebar = visiblePercentage > 0 ? (sidebarPercentage / visiblePercentage) * width : width * 0.3
      sidebarPx = Math.max(MIN_SIDEBAR_PX, Math.min(width - MIN_EDITOR_PX, Math.round(rawSidebar)))
    } else if (sidebarHiddenState && !outputHiddenState) {
      const visiblePercentage = outputPercentage + editorPercentage
      const rawOutput = visiblePercentage > 0 ? (outputPercentage / visiblePercentage) * width : width * 0.4
      outputPx = Math.max(MIN_OUTPUT_PX, Math.min(width - MIN_EDITOR_PX, Math.round(rawOutput)))
    }
  } else if (isLeftOverlay.value && !isRightOverlay.value && !outputHiddenState) {
    const visiblePercentage = outputPercentage + editorPercentage
    const rawOutput = visiblePercentage > 0 ? (outputPercentage / visiblePercentage) * width : width * 0.4
    outputPx = Math.max(MIN_OUTPUT_PX, Math.min(width - MIN_EDITOR_PX, Math.round(rawOutput)))
  }

  return { sidebarPx, outputPx }
})

const sidebarWidth = computed(() => layout.value.sidebarPx)
const outputWidth = computed(() => layout.value.outputPx)

function resolvePanelConflict(justOpened: 'sidebar' | 'output') {
  const width = windowWidth.value
  if (width <= 0) return

  if (justOpened === 'sidebar' && isLeftOverlay.value) return
  if (justOpened === 'output' && isRightOverlay.value) return

  if (justOpened === 'sidebar' && !outputHidden.value) {
    const sidebarTarget = Math.max(MIN_SIDEBAR_PX, Math.round((sidebarPct.value / 100) * width))
    if (width - sidebarTarget - MIN_EDITOR_PX < MIN_OUTPUT_PX) {
      outputHidden.value = true
      outputClosedByForce.value = true
    }
  } else if (justOpened === 'output' && !sidebarHidden.value) {
    const outputTarget = Math.max(MIN_OUTPUT_PX, Math.round((outputPct.value / 100) * width))
    if (width - outputTarget - MIN_EDITOR_PX < MIN_SIDEBAR_PX) {
      sidebarHidden.value = true
      sidebarClosedByForce.value = true
    }
  }
}

watch(sidebarHidden, (hidden) => { if (!hidden) resolvePanelConflict('sidebar') })
watch(outputHidden, (hidden) => { if (!hidden) resolvePanelConflict('output') })

export const useEditorLayoutStore = () => {
  const closeSidebarByForce = () => { sidebarHidden.value = true; sidebarClosedByForce.value = true }
  const closeOutputByForce = () => { outputHidden.value = true; outputClosedByForce.value = true }
  const closeSidebar = () => { sidebarHidden.value = true }
  const closeOutput = () => { outputHidden.value = true }
  const closeSidebarByUser = () => { sidebarHidden.value = true; sidebarClosedByForce.value = false }
  const closeOutputByUser = () => { outputHidden.value = true; outputClosedByForce.value = false }
  const openSidebar = () => { sidebarHidden.value = false }
  const openOutput = () => { outputHidden.value = false }

  const toggleSidebarByUser = () => sidebarHidden.value ? openSidebar() : closeSidebarByUser()
  const toggleOutputByUser = () => outputHidden.value ? openOutput() : closeOutputByUser()

  const closeOverlaysByForce = () => { closeSidebarByForce(); closeOutputByForce() }
  const closeOverlays = () => {
    if (isLeftOverlay.value) closeSidebar()
    if (isRightOverlay.value) closeOutput()
  }

  function setSidebarWidthFromPixels(pixelWidth: number) {
    const width = windowWidth.value
    if (width <= 0 || isLeftOverlay.value) return

    if (!outputHidden.value) {
      let targetSidebarPx = Math.max(MIN_SIDEBAR_PX, pixelWidth)
      const currentOutputPx = Math.max(MIN_OUTPUT_PX, layout.value.outputPx)

      if (targetSidebarPx + currentOutputPx + MIN_EDITOR_PX > width) {
        const remainingForOutput = width - targetSidebarPx - MIN_EDITOR_PX

        if (remainingForOutput < MIN_OUTPUT_PX) {
          outputHidden.value = true
          outputClosedByForce.value = true
          targetSidebarPx = Math.min(targetSidebarPx, width - MIN_EDITOR_PX)
          sidebarPct.value = (targetSidebarPx / width) * (100 - outputPct.value)
        } else {
          sidebarPct.value = (targetSidebarPx / width) * 100
          outputPct.value = (remainingForOutput / width) * 100
        }
      } else {
        sidebarPct.value = (targetSidebarPx / width) * 100
      }
    } else {
      const maxSidebarPx = width - MIN_EDITOR_PX
      const targetSidebarPx = Math.max(MIN_SIDEBAR_PX, Math.min(maxSidebarPx, pixelWidth))
      sidebarPct.value = (targetSidebarPx / width) * (100 - outputPct.value)
    }
  }

  function setOutputWidthFromPixels(pixelWidth: number) {
    const width = windowWidth.value
    if (width <= 0 || isRightOverlay.value) return

    if (!sidebarHidden.value) {
      let targetOutputPx = Math.max(MIN_OUTPUT_PX, pixelWidth)
      const currentSidebarPx = Math.max(MIN_SIDEBAR_PX, layout.value.sidebarPx)

      if (targetOutputPx + currentSidebarPx + MIN_EDITOR_PX > width) {
        const remainingForSidebar = width - targetOutputPx - MIN_EDITOR_PX

        if (remainingForSidebar < MIN_SIDEBAR_PX) {
          sidebarHidden.value = true
          sidebarClosedByForce.value = true
          targetOutputPx = Math.min(targetOutputPx, width - MIN_EDITOR_PX)
          outputPct.value = (targetOutputPx / width) * (100 - sidebarPct.value)
        } else {
          outputPct.value = (targetOutputPx / width) * 100
          sidebarPct.value = (remainingForSidebar / width) * 100
        }
      } else {
        outputPct.value = (targetOutputPx / width) * 100
      }
    } else {
      const maxOutputPx = width - MIN_EDITOR_PX
      const targetOutputPx = Math.max(MIN_OUTPUT_PX, Math.min(maxOutputPx, pixelWidth))
      outputPct.value = (targetOutputPx / width) * (100 - sidebarPct.value)
    }
  }

  function updateBreakpoints(currentWidth: number) {
    const applyBreakpoint = (
      isSmall: boolean,
      hiddenState: typeof sidebarHidden,
      forceState: typeof sidebarClosedByForce
    ) => {
      if (isSmall) {
        if (!hiddenState.value) { hiddenState.value = true; forceState.value = true }
      } else if (hiddenState.value && forceState.value) {
        hiddenState.value = false; forceState.value = false
      }
    }

    applyBreakpoint(currentWidth < MIN_THREE_PANEL_PX, sidebarHidden, sidebarClosedByForce)
    applyBreakpoint(currentWidth < MIN_TWO_PANEL_PX, outputHidden, outputClosedByForce)

    if (Math.abs(savedScreenWidth.value - currentWidth) > currentWidth * 0.15) {
      sidebarPct.value = DEFAULT_SIDEBAR_PCT
      outputPct.value = DEFAULT_OUTPUT_PCT
    }
    savedScreenWidth.value = currentWidth
  }

  function handleResize() {
    if (typeof window === 'undefined') return
    windowWidth.value = window.innerWidth
    updateBreakpoints(window.innerWidth)
  }

  function initPanelWidths() {
    if (typeof window === 'undefined') return
    const currentWidth = window.innerWidth
    windowWidth.value = currentWidth

    if (currentWidth < MIN_TWO_PANEL_PX) {
      sidebarHidden.value = true; sidebarClosedByForce.value = true
      outputHidden.value = true; outputClosedByForce.value = true
    } else if (currentWidth < MIN_THREE_PANEL_PX) {
      sidebarHidden.value = true; sidebarClosedByForce.value = true
    } else {
      const widthDiff = Math.abs((savedScreenWidth.value || currentWidth) - currentWidth)
      if (savedScreenWidth.value === 0 || widthDiff > currentWidth * 0.15) {
        sidebarPct.value = DEFAULT_SIDEBAR_PCT
        outputPct.value = DEFAULT_OUTPUT_PCT
      }
    }
    savedScreenWidth.value = currentWidth
  }

  const toggleLog = () => { showLog.value = !showLog.value }

  return {
    sidebarHidden,
    outputHidden,
    sidebarClosedByForce,
    outputClosedByForce,
    showLog,
    sidebarPct,
    outputPct,
    sidebarWidth,
    outputWidth,
    isLeftOverlay,
    isRightOverlay,
    savedScreenWidth,
    initPanelWidths,
    handleResize,
    setSidebarWidthFromPixels,
    setOutputWidthFromPixels,
    toggleSidebarByUser,
    toggleOutputByUser,
    closeSidebarByUser,
    closeOutputByUser,
    closeSidebar,
    closeOutput,
    openSidebar,
    openOutput,
    closeSidebarByForce,
    closeOutputByForce,
    closeOverlaysByForce,
    closeOverlays,
    toggleLog,
  }
}

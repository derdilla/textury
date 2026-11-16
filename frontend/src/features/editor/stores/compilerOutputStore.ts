import type { Compiler } from '@/api/types'
import { reactive } from 'vue'

export type LogType = 'ok' | 'warn' | 'err'

interface CompilerOutputState {
  lastCompiledUrl: string
  logType: LogType
  log: string | null
  compilerPref: Compiler | null
}

const initialState: CompilerOutputState = {
  lastCompiledUrl: '/samples/doc.pdf',
  logType: 'ok',
  log: null,
  compilerPref: null,
}

const state = reactive<CompilerOutputState>({ ...initialState })

export const useCompilerOutputStore = () => {
  function setLog(log: string | null, type: LogType = 'ok') {
    state.log = log;
    // Only increment level, decrement only on reset
    if (log === null) state.logType = initialState.logType;
    if (state.logType === 'ok' && type === 'warn') state.logType = type;
    if (type === 'err') state.logType = type;
  }

  function setCompiledPdf(url: string) {
    state.lastCompiledUrl = url
  }

  function resetCompilerState() {
    Object.assign(state, initialState)
  }

  return {
    state,
    setLog,
    setCompiledPdf,
    resetCompilerState,
  }
}

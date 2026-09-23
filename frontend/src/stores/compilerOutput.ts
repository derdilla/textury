import { reactive } from 'vue'

export type LogType = 'ok' | 'warn' | 'err'
export type Compiler = 'Typst' | 'PdfLatex' | 'XeLatex' | 'LuaLatex'

interface CompilerOutputState {
  lastCompiledUrl: string
  logType: LogType
  log: string | null
  compilerPref: Compiler | null
}

const initialState: CompilerOutputState = {
  lastCompiledUrl: '/sampleDoc.pdf',
  logType: 'ok',
  log: null,
  compilerPref: null,
}

const state = reactive<CompilerOutputState>({ ...initialState })

export const useCompilerOutputStore = () => {
  function setLog(log: string | null, type: LogType = 'ok') {
    state.log = log
    state.logType = type
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
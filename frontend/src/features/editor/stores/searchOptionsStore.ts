import { useStorage } from '@vueuse/core'

export const useSearchOptionsStore = () => {
  const caseSensitive = useStorage<boolean>('search_case_sensitive', false)
  const wholeWord = useStorage<boolean>('search_whole_word', false)
  const regexp = useStorage<boolean>('search_regexp', false)

  function setCaseSensitive(value: boolean) {
    caseSensitive.value = value
  }

  function setWholeWord(value: boolean) {
    wholeWord.value = value
  }

  function setRegexp(value: boolean) {
    regexp.value = value
  }

  return {
    caseSensitive,
    wholeWord,
    regexp,
    setCaseSensitive,
    setWholeWord,
    setRegexp,
  }
}

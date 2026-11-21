<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onUnmounted } from "vue";
import type { EditorView } from "@codemirror/view";
import FloatingElement from "@/shared/components/primitives/FloatingElement.vue";
import { useSearchOptionsStore } from "@/features/editor/stores/searchOptionsStore";
import {
  getSearchQuery,
  setSearchQuery,
  SearchQuery,
  findNext,
  findPrevious,
  selectMatches,
  replaceNext,
  replaceAll,
  closeSearchPanel
} from "@codemirror/search";

const props = defineProps<{
  view: EditorView;
}>();

const searchOptionsStore = useSearchOptionsStore();

const initialQuery = getSearchQuery(props.view.state);

const searchText = ref(initialQuery.search);
const replaceText = ref(initialQuery.replace);

const caseSensitive = searchOptionsStore.caseSensitive;
const regexp = searchOptionsStore.regexp;
const wholeWord = searchOptionsStore.wholeWord;

const showOptions = ref(false);
const showReplace = ref(false);

const inputField = ref<HTMLInputElement | null>(null);
const replaceField = ref<HTMLInputElement | null>(null);

const currentMatch = ref(0);
const totalMatches = ref(0);

onMounted(async () => {
  await nextTick();

  inputField.value?.focus();
  inputField.value?.select();

  document.addEventListener("keydown", handleKeydown);
  updateMatchCount();
});

onUnmounted(() => {
  document.removeEventListener("keydown", handleKeydown);
});

function syncSearchQuery() {
  const currentQuery = getSearchQuery(props.view.state);

  const query = new SearchQuery({
    search: searchText.value,
    replace: replaceText.value,
    caseSensitive: caseSensitive.value,
    regexp: regexp.value,
    wholeWord: wholeWord.value,
    literal: currentQuery.literal
  });

  if (query.eq(currentQuery)) return;

  props.view.dispatch({
    effects: setSearchQuery.of(query)
  });
}

function updateMatchCount() {
  const query = getSearchQuery(props.view.state);

  if (!query.search) {
    currentMatch.value = 0;
    totalMatches.value = 0;
    return;
  }

  const cursor = query.getCursor(props.view.state.doc);
  const selection = props.view.state.selection.main;

  let count = 0;
  let activeMatch = 0;

  for (let match = cursor.next(); !match.done; match = cursor.next()) {
    count++;

    if (
      match.value.from === selection.from &&
      match.value.to === selection.to
    ) {
      activeMatch = count;
    }
  }

  totalMatches.value = count;
  currentMatch.value = activeMatch;
}

watch(
  [searchText, replaceText, caseSensitive, regexp, wholeWord],
  () => {
    syncSearchQuery();
    updateMatchCount();
  },
  { immediate: true }
);

watch(
  () => props.view,
  () => {
    syncSearchQuery();
    updateMatchCount();
  }
);

function next() {
  findNext(props.view);
  updateMatchCount();
}

function previous() {
  findPrevious(props.view);
  updateMatchCount();
}

function selectAll() {
  selectMatches(props.view);
  updateMatchCount();
}

function replace() {
  replaceNext(props.view);
  updateMatchCount();
}

function replaceEverything() {
  replaceAll(props.view);
  updateMatchCount();
}

async function toggleReplace() {
  showReplace.value = !showReplace.value;

  await nextTick();

  if (showReplace.value) {
    replaceField.value?.focus();
  } else {
    inputField.value?.focus();
  }
}

function close() {
  props.view.focus();
  closeSearchPanel(props.view);
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key !== "Escape") return;

  event.preventDefault();
  event.stopPropagation();
  close();
}
</script>

<template>
  <div class="search-panel bottom-margin">
    <!-- Find -->
    <div class="row search-row">
      <!-- Toggle Replace -->
      <FloatingElement hover align="center">
        <template #button>
          <button
            class="square transparent surface-container-high"
            :aria-label="showReplace ? 'Hide replace' : 'Show replace'"
            @click="toggleReplace"
          >
            <i>{{ showReplace ? "expand_more" : "chevron_right" }}</i>
          </button>
        </template>
        <template #menu>
          <span class="tooltip bottom active">
            {{ showReplace ? "Hide replace" : "Show replace" }}
          </span>
        </template>
      </FloatingElement>

      <div class="search-box">
        <div class="field label border no-round small no-margin input-field">
          <input
            ref="inputField"
            v-model="searchText"
            type="text"
            class="search-input custom-placeholder mono-text"
            placeholder=""
          >
          <label class="search-label">Find</label>
        </div>

        <span v-if="searchText" class="match-count mono-text">
          {{ currentMatch }}/{{ totalMatches }}
        </span>
      </div>

      <!-- Navigation -->
      <div class="group-button row">
        <FloatingElement hover align="center">
          <template #button>
            <button class="square fill left-group-button" @click="previous">
              <i>keyboard_arrow_up</i>
            </button>
          </template>
          <template #menu>
            <span class="tooltip bottom active">Previous match</span>
          </template>
        </FloatingElement>

        <FloatingElement hover align="center">
          <template #button>
            <button class="square fill right-group-button" @click="next">
              <i>keyboard_arrow_down</i>
            </button>
          </template>
          <template #menu>
            <span class="tooltip bottom active">Next match</span>
          </template>
        </FloatingElement>
      </div>

      <!-- Select all -->
      <!--
      <FloatingElement hover align="center">
        <template #button>
          <button
            class="square fill"
            @click="selectAll"
          >
            <i>select_all</i>
          </button>
        </template>
        <template #menu>
          <span class="tooltip bottom active">Select all matches</span>
        </template>
      </FloatingElement>
      -->

      <!-- Search options -->
      <FloatingElement hover align="center">
        <template #button>
          <FloatingElement align="left">
            <template #button>
              <button
                class="square fill"
                :class="{ 'secondary-container': showOptions }"
                @click="showOptions = !showOptions"
              >
                <i>tune</i>
              </button>
            </template>

            <template #menu>
              <menu class="dropdown no-wrap no-round active surface-container">
                <span
                  class="dropdown-title grey-text left-padding"
                  style="padding-top: 8px; padding-bottom: 5px;"
                >
                  Search options
                </span>

                <li><hr /></li>

                <button
                  class="transparent no-round left-align"
                  @click.stop="caseSensitive = !caseSensitive"
                >
                  <i class="small">text_fields</i>
                  Match case

                  <label class="switch" @click.stop>
                    <input type="checkbox" v-model="caseSensitive">
                    <span></span>
                  </label>
                </button>

                <button
                  class="transparent no-round left-align"
                  @click.stop="wholeWord = !wholeWord"
                >
                  <i class="small">match_word</i>
                  Whole word

                  <label class="switch" @click.stop>
                    <input type="checkbox" v-model="wholeWord">
                    <span></span>
                  </label>
                </button>

                <button
                  class="transparent no-round left-align"
                  @click.stop="regexp = !regexp"
                >
                  <i class="small">regular_expression</i>
                  Regular expression

                  <label class="switch" @click.stop>
                    <input type="checkbox" v-model="regexp">
                    <span></span>
                  </label>
                </button>
              </menu>
            </template>
          </FloatingElement>
        </template>

        <template #menu>
          <span class="tooltip bottom active">Search options</span>
        </template>
      </FloatingElement>

      <!-- Close -->
      <FloatingElement hover align="center">
        <template #button>
          <button class="square transparent surface-container-high" @click="close">
            <i>close</i>
          </button>
        </template>
        <template #menu>
          <span class="tooltip bottom active">Close search</span>
        </template>
      </FloatingElement>
    </div>

    <!-- Replace -->
    <div v-if="showReplace && !props.view.state.readOnly" class="row replace-row">
      <div class="replace-indent"></div>

      <div class="field label border no-round small no-margin input-field search-box">
        <input
          ref="replaceField"
          v-model="replaceText"
          type="text"
          class="search-input custom-placeholder mono-text"
          placeholder=""
        >
        <label class="search-label">Replace</label>
      </div>

      <FloatingElement hover align="center">
        <template #button>
          <button class="square fill" @click="replace">
            <i>find_replace</i>
          </button>
        </template>
        <template #menu>
          <span class="tooltip bottom active">Replace current match</span>
        </template>
      </FloatingElement>

      <FloatingElement hover align="center">
        <template #button>
          <button class="square fill" @click="replaceEverything">
            <i>done_all</i>
          </button>
        </template>
        <template #menu>
          <span class="tooltip bottom active">Replace all matches</span>
        </template>
      </FloatingElement>

      <div class="replace-indent"></div>
    </div>
  </div>
</template>

<style scoped>
.search-panel {
  display: flex;
  flex-direction: column;
}

.search-row,
.replace-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 1rem;
  width: 100%;
}

.group-button {
  gap: 0.15rem;
}

.left-group-button {
  border-top-right-radius: 0 !important;
  border-bottom-right-radius: 0 !important;
}

.right-group-button {
  border-top-left-radius: 0 !important;
  border-bottom-left-radius: 0 !important;
}

.replace-indent {
  flex: 0 0 2.5rem;
}

.search-box {
  position: relative;
  flex: 1;
  min-width: 0;
}

.search-box .field {
  width: 100%;
}

.search-input {
  padding-inline-start: 1rem !important;
  padding-inline-end: 3.5rem !important;
}

.match-count {
  position: absolute;
  right: 1rem;
  top: 50%;
  transform: translateY(-50%);
  font-size: 0.75rem;
  line-height: 1;
  color: var(--on-surface-variant);
  pointer-events: none;
  user-select: none;
}

.search-input:is(:focus,[placeholder]:not(:placeholder-shown),.active) {
  clip-path: polygon(-2% -2%, 0.75rem -2%, 0.75rem .5rem, calc(100% - 2rem) .5rem, calc(100% - 2rem) -2%, 102% -2%, 102% 102%, -2% 102%) !important;
}

.search-input:focus {
  padding-inline-start: 0.9375rem !important;
}

.search-box label {
  margin-left: -0.5rem !important;
}

.search-label {
  margin-left: -0.5rem !important;
}

.custom-placeholder::placeholder {
  opacity: 0 !important;
  transition: opacity 0.2s ease;
}

.custom-placeholder:focus::placeholder {
  opacity: 1 !important;
}

.dropdown hr {
  margin: 3px;
}

.dropdown i {
  margin-right: 0.5rem;
}

.dropdown {
  gap: 4px;
  box-shadow: none !important;
}

.dropdown .switch {
  margin-left: auto;
  padding-left: 0.5rem;
  margin-right: -0.5rem;
}
</style>

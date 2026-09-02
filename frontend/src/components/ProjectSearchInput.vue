<script setup lang="ts">
import { ref, watch, nextTick } from "vue";

const props = defineProps<{
  modelValue?: string;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
}>();

const query = ref(props.modelValue ?? "");
const inputField = ref<HTMLInputElement | null>(null);
const isExpanded = ref(false);

watch(
  () => props.modelValue,
  (newVal) => {
    query.value = newVal ?? "";
    if (newVal) {
      isExpanded.value = true;
    }
  }
);

function onInput(event: Event) {
  const value = (event.target as HTMLInputElement).value;
  query.value = value;
  emit("update:modelValue", value);
}

async function expandSearch() {
  isExpanded.value = true;
  await nextTick();
  inputField.value?.focus();
}

function handleClose() {
  query.value = "";
  emit("update:modelValue", "");
  isExpanded.value = false;
  if (inputField.value) {
    inputField.value.blur();
  }
}

function handleBlur() {
  if (!query.value.trim()) {
    isExpanded.value = false;
  }
}
</script>

<template>
  <div class="field border small no-round search-box prefix label"
    :class="{ 'is-expanded': isExpanded || query.length > 0 }">
    <i class="search-icon">search</i>
    <input ref="inputField" type="text" :value="query" @input="onInput" @focus="expandSearch" @blur="handleBlur" @keydown.esc.prevent="handleClose" placeholder="" class="search-input" />
    <label class="search-label">Search</label>

    <button class="square transparent small absolute right close-button" type="button" @mousedown.prevent="handleClose" title="Close search">
      <i class="small">close</i>
    </button>

    <button class="absolute right square border search-trigger" type="button" tabindex="-1" @mousedown.prevent="expandSearch">
      <i>search</i>
    </button>
  </div>
</template>

<style scoped>
.field.border > :is(input, textarea, select) {
  border-color: var(--outline-variant);
}

.field.label.border:not(.fill) > label:after {
  border-color: var(--outline-variant);
}

.search-box {
  width: 2.5rem;
  min-width: 2.5rem;
  transition: width var(--speed2), background-color var(--speed2);
  white-space: nowrap;
  clip-path: inset(-50px 0px -50px 0px);

  &::after {
    content: "";
    position: absolute;
    inset: 0;
    background-image: radial-gradient(circle, var(--secondary-container) 1%, transparent 1%);
    background-position: center;
    background-size: 100%;
    opacity: 0;
    transition: background-size 0.4s, opacity 0.2s;
    border-radius: inherit;
  }

  &:is(:hover, :focus-visible)::after {
    background-size: 22500%;
    opacity: 0.3;
    z-index: -1;
  }

  input, 
  label {
    opacity: 0;
    transition: opacity var(--speed1);
    pointer-events: none;
  }
}

.search-icon {
  min-width: 2rem;
  display: flex;
  justify-content: center;
  opacity: 0;
  scale: 0.9;
  margin-left: -11px !important;
}

.search-label {
  margin-left: -11px !important;
}

.search-input {
  /* Overwrite default BeerCSS paddings that come with prefix and suffix. */
  padding-inline-start: 2.38rem !important;
  padding-inline-end: 2.38rem !important;
}

.search-input:focus {
  /* Offset of 0.0625 is needed to stop the text from jumping when focusing. Derived from BeerCSS. */
  padding-inline-start: 2.3175rem !important;
  /* Slightly changed mask from BeerCSS. Changed 1.5rem to 0.55rem. */
  clip-path: polygon(-2% -2%, 0.55rem -2%, 0.55rem .5rem, calc(100% - 2rem) .5rem, calc(100% - 2rem) -2%, 102% -2%, 102% 102%, -2% 102%) !important;
}

.search-box:not(:focus-within):has(input:is(:placeholder-shown)) label {
  margin-left: -10px !important;
}

.close-button {
  opacity: 0;
  pointer-events: none;
  transition: opacity var(--speed1);
  z-index: 2;
  top: 50%;
  transform: translateY(-50%);
  right: 0.25rem;
}

@media (min-width: 801px) {
  .search-box:is(:hover, :focus-within, .is-expanded) {
    width: 15rem;
    cursor: text;

    input, label, .search-icon {
      opacity: 1;
      pointer-events: auto;
    }

    .search-trigger {
      opacity: 0;
      pointer-events: none;
    }
  }

  .search-box:is(:focus-within, .is-expanded) .close-button {
    opacity: 1;
    pointer-events: auto;
  }
}

@media (max-width: 800px) {
  .search-box:is(:focus-within, .is-expanded) {
    position: absolute;
    top: 0;
    bottom: 0;
    left: 1rem;
    right: 1rem;
    width: calc(100% - (1rem * 2)) !important;
    min-width: calc(100% - (1rem * 2)) !important;
    height: 100% !important;
    z-index: 100;
    background-color: var(--surface);

    input, label, .search-icon, .close-button {
      opacity: 1;
      pointer-events: auto;
    }

    .search-trigger {
      opacity: 0;
      pointer-events: none;
    }
  }
}
</style>
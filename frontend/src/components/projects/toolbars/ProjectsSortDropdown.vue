<script setup lang="ts">
import { ref } from "vue";
import FloatingElement from "@/components/general/FloatingElement.vue";
import { Sorting } from "@/services/projectController.ts";

const sorting = defineModel<Sorting>("sorting", { required: true });

const outerFloating = ref<InstanceType<typeof FloatingElement> | null>(null);

function handleButtonClick() {
  if (outerFloating.value) {
    outerFloating.value.isOpen = false;
  }
}

function selectSorting(val: Sorting) {
  sorting.value = val;
}
</script>

<template>
  <FloatingElement ref="outerFloating" hover align="center">
    <template #button>
      <FloatingElement align="left">
        <template #button>
          <button tabindex="0" class="square border" @click="handleButtonClick">
            <i>sort</i>
          </button>
        </template>
        <template #menu>
          <menu class="dropdown no-wrap no-round active surface-container shadow">
            <span class="dropdown-title grey-text left-padding" style="padding-top: 8px; padding-bottom: 5px;">
              Sort by
            </span>
            <li><hr /></li>
            <button
              class="transparent no-round left-align"
              :class="{ 'secondary-container': sorting === Sorting.name }"
              @click="selectSorting(Sorting.name)"
            >
              <i class="small">sort_by_alpha</i> Name
            </button>
            <button
              class="transparent no-round left-align"
              :class="{ 'secondary-container': sorting === Sorting.dateEdited }"
              @click="selectSorting(Sorting.dateEdited)"
            >
              <i class="small">update</i> Date edited
            </button>
            <button
              class="transparent no-round left-align"
              :class="{ 'secondary-container': sorting === Sorting.dateCreated }"
              @click="selectSorting(Sorting.dateCreated)"
            >
              <i class="small">calendar_today</i> Date created
            </button>
          </menu>
        </template>
      </FloatingElement>
    </template>
    <template #menu>
      <span class="tooltip bottom active">Sorting</span>
    </template>
  </FloatingElement>
</template>

<style scoped>
.dropdown hr {
  margin: 3px;
}
.dropdown i {
  margin-right: 0.5rem;
}
.dropdown {
  gap: 4px;
}
.dropdown-title {
  margin-bottom: -2px;
}
</style>
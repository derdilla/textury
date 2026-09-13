<script setup lang="ts">
import { reactive, computed, provide } from "vue";
import { createRouter, createMemoryHistory, routerKey, routeLocationKey, type RouteRecordRaw } from "vue-router";

const props = defineProps<{
  routes: RouteRecordRaw[];
  initialRoute?: string;
}>();

const memoryRouter = createRouter({
  history: createMemoryHistory(),
  routes: props.routes,
});

if (props.initialRoute) {
  memoryRouter.push(props.initialRoute);
}

// Reactive Bridge in order to be able to use route.params like normal.
const reactiveRoute = reactive({
  path: computed(() => memoryRouter.currentRoute.value.path),
  params: computed(() => memoryRouter.currentRoute.value.params),
  query: computed(() => memoryRouter.currentRoute.value.query),
  hash: computed(() => memoryRouter.currentRoute.value.hash),
  fullPath: computed(() => memoryRouter.currentRoute.value.fullPath),
  matched: computed(() => memoryRouter.currentRoute.value.matched),
  meta: computed(() => memoryRouter.currentRoute.value.meta),
  name: computed(() => memoryRouter.currentRoute.value.name),
  redirectedFrom: computed(() => memoryRouter.currentRoute.value.redirectedFrom),
});

provide(routerKey, memoryRouter);
provide(routeLocationKey, reactiveRoute);
</script>

<template>
  <slot />
</template>
<script setup lang="ts">
defineProps(['toggleMenu'])
</script>

<template>
  <nav class="left panel round">
    <button @click="toggleMenu" class="transparent circle extra small-margin">
      <i>menu</i>
    </button>
    <h5 class="small no-padding no-margin">Project Title</h5>
    <ul>
      <li>cargo.toml</li>
      <li>cargo.lock</li>
      <li>
        <details open class="no-padding no-margin">
          <summary class="no-margin no-padding">src/</summary>
          <ul>
            <li>main.rs</li>
            <li>lib.rs</li>
          </ul>
        </details>
      </li>
    </ul>
    <div class="max"></div>
  </nav>
</template>

// TODO: figure out how to style nested lists nicely
<style scoped>
li {
  list-style: none;
  align-self: start;
  padding: 0;
  margin: 0;
  line-height: 1.5;
}

ul {
  margin: 0;
  padding: 0;
}

details[open] {
  margin: 0;
  padding: 0;
}

</style>
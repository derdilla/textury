<script setup lang="ts">
import T_dark from '@/assets/orbit/T_dark.svg';
import Y from '@/assets/orbit/Y.svg';
import R from '@/assets/orbit/R.svg';
import U_dark from '@/assets/orbit/U_dark.svg';
import X from '@/assets/orbit/X.svg';
import E from '@/assets/orbit/E.svg';
import logo_dark from '@/assets/logo_dark.svg';

import T_light from '@/assets/orbit/T_light.svg';
import U_light from '@/assets/orbit/U_light.svg';
import logo_light from '@/assets/logo_light.svg';

const squareSvgs = [
  { dark: T_dark, light: T_light },
  { dark: Y, light: Y },
  { dark: R, light: R },
  { dark: U_dark, light: U_light },
  { dark: X, light: X },
  { dark: E, light: E },
];
</script>

<template>
  <div class="artwork-view">
    <img :src="logo_dark" alt="Textury Logo" class="center-logo dark-only"/>
    <img :src="logo_light" alt="Textury Logo" class="center-logo light-only"/>

    <svg viewBox="0 0 100 100" class="orbit-layer">
      <circle cx="50" cy="50" r="49" fill="none" stroke="var(--orbit-stroke)" stroke-width="1.5" stroke-dasharray="20 25" vector-effect="non-scaling-stroke"/>
    </svg>

    <div class="orbit-layer">
      <div v-for="(svg, i) in squareSvgs" :key="i" class="square-pivot" :style="{ '--i': i }">
        <div class="orbit-square" :style="{ backgroundColor: `var(--square-bg-${i})` }">
          <img :src="svg.dark" class="square-img dark-only"/>
          <img :src="svg.light" class="square-img light-only"/>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.artwork-view {
  --orbit-speed: 40s;
  width: 540px;
  height: 540px;
  position: relative;
  display: flex;
  justify-content: center;
  align-items: center;

  --orbit-stroke: #8c1943;
  --square-bg-0: #ff3954;
  --square-bg-1: #8c1943;
  --square-bg-2: #ff9eb1;
  --square-bg-3: #ff3954;
  --square-bg-4: #8c1943;
  --square-bg-5: #ff9eb1;
}
.dark-only { display: none; }
.light-only { display: block; }

.center-logo {
  position: absolute;
  z-index: 10;
  width: 340px;
}
.orbit-layer {
  position: absolute;
  inset: 0;
  overflow: visible;
  animation: rotate-cw var(--orbit-speed) linear infinite;
}
.square-pivot {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: rotate(calc(60deg * var(--i))) translateY(-270px) rotate(calc(-60deg * var(--i)));
}
.orbit-square {
  width: 90px;
  height: 90px;
  border-radius: 24px;
  overflow: hidden;
  animation: rotate-ccw var(--orbit-speed) linear infinite;
}
.square-img {
  width: 100%;
  height: 100%;
}
@keyframes rotate-cw { to { transform: rotate(360deg); } }
@keyframes rotate-ccw {
  from { transform: translate(-50%, -50%) rotate(0deg); }
  to { transform: translate(-50%, -50%) rotate(-360deg); }
}

.dark .artwork-view {
  --orbit-stroke: #ff9eb1;
  --square-bg-0: #feebdd;
  --square-bg-1: #8c1943;
  --square-bg-2: #ff9eb1;
  --square-bg-3: #feebdd;
  --square-bg-4: #8c1943;
  --square-bg-5: #ff9eb1;
}
.dark .dark-only { display: block; }
.dark .light-only { display: none; }

@media (prefers-reduced-motion: reduce) {
  .orbit-layer,
  .orbit-square {
    animation-play-state: paused;
  }
}
</style>
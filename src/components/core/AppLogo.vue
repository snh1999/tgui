<script setup lang="ts">
  import logo from "@/assets/logo.svg";

  defineProps<{
    scale?: number;
  }>();
</script>

<template>
  <div
    class="my-2 flex items-center justify-center gap-3"
    :style="{scale: scale?? 1}"
  >
    <img class="h-8 w-8" alt="logo" :src="logo">
    <div class="text-xl font-semibold tracking-tighter">TGUI</div>
  </div>
</template>

<style scoped>
</style>

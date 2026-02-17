<script setup lang="ts">
  /** use it under the container element, some fields may remain in focus otherwise **/
  import { Spinner } from "@/components/ui/spinner";
</script>

<template>
  <div
    class="absolute inset-0 flex items-center justify-center bg-background/50 z-50"
  >
    <Spinner class="h-8 w-8" />
  </div>
</template>

<style scoped>
</style>

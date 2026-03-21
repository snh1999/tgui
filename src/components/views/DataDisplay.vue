<script setup lang="ts">
  defineProps<{ view: "grid" | "list" | "table" }>();
</script>

<template>
  <div v-if="view === 'list'" class="flex-1 overflow-y-auto ">
    <slot name="list" />
  </div>

  <div
    v-else-if="view === 'grid'"
    class="flex-1 overflow-y-auto grid gap-2.5"
    style="grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); align-content: start;"
  >
    <slot name="grid" />
  </div>

  <div v-else-if="view === 'table'">
    <slot name="table" />
  </div>
</template>

<style scoped>
</style>

<script setup lang="ts">
</script>

<template>
  <main>
    <slot name="header" />
    <div class="flex-1 px-6 md:p-8 lg:px-10">
      <slot />
    </div>
  </main>
</template>

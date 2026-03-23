<script setup lang="ts">
  defineProps<{
    title: string;
    description: string;
    destructive?: boolean;
  }>();
</script>

<template>
  <div>
    <h2
      class="text-xl font-semibold"
      :class="destructive? 'text-destructive' : ''"
    >
      {{ title }}
    </h2>
    <p class="text-sm py-2.5 text-muted-foreground">{{ description }}</p>

    <div
      class="flex flex-col bg-card text-card-foreground border overflow-hidden"
    >
      <slot />
    </div>
  </div>
</template>

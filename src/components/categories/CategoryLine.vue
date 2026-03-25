<script setup lang="ts">
  import * as Icons from "lucide-vue-next";
  import { capitalize, computed } from "vue";
  import { ICategory } from "@/lib/api/api.types.ts";

  const props = defineProps<{ category: ICategory; compact?: boolean }>();

  const iconComponent = computed(() => {
    if (!props.category?.icon) return null;
    return (Icons as Record<string, unknown>)[props.category.icon] ?? null;
  });
</script>

<template>
  <component
    :is="iconComponent"
    v-if="iconComponent"
    :size="16"
    :style="{ color: category.color }"
  />
  <span
    v-else-if="!compact"
    class="inline-block rounded-md shrink-0  w-4 h-4"
    :style="{ backgroundColor: category.color?? 'grey' }"
  />
  {{ capitalize(category.name) }}
</template>

<script setup lang="ts">
  // biome-ignore lint/performance/noNamespaceImport: <only way to show icons>
  import * as Icons from "lucide-vue-next";
  import { capitalize, computed } from "vue";
  import { ICategory } from "@/lib/api/api.types.ts";

  /** using ICategory instead of custom type as this has common values of IGroup as well*/
  const props = defineProps<{
    element: Omit<ICategory, "createdAt" | "id">;
    compact?: boolean;
  }>();

  const iconComponent = computed(() => {
    if (!props.element?.icon) {
      return null;
    }
    return (Icons as Record<string, unknown>)[props.element.icon] ?? null;
  });
</script>

<template>
  <component
    :is="iconComponent"
    v-if="iconComponent"
    :size="16"
    :style="{ color: element.color }"
  />
  <span
    v-else-if="!compact"
    class="inline-block rounded-md shrink-0  w-4 h-4"
    :style="{ backgroundColor: element.color?? 'grey' }"
  />
  {{ capitalize(element.name) }}
</template>

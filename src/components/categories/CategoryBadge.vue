<script setup lang="ts">
  import { Badge } from "@/components/ui/badge";
  import { useGetCategory } from "@/lib/api/composables/categories.ts";
  import CategoryLine from "@/components/categories/CategoryLine.vue";

  const props = defineProps<{ categoryId: number }>();
  const { data: category } = useGetCategory(props.categoryId);
</script>

<template>
  <Badge
    v-if="category"
    variant="outline"
    class="h-6 mx-2 border-none gap-1.5"
    :style="{ backgroundColor: `${category.color}25` }"
  >
    <CategoryLine :category="category" compact />
  </Badge>
</template>

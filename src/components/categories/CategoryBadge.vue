<script setup lang="ts">
  import GroupCategoryLine from "@/components/shared/GroupCategoryLine.vue";
  import { Badge } from "@/components/ui/badge";
  import { useGetCategory } from "@/lib/api/composables/categories.ts";

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
    <GroupCategoryLine :element="category" compact />
  </Badge>
</template>

<script setup lang="ts">
  import { Badge } from "@/components/ui/badge";
  import { useGetCategory } from "@/lib/api/composables/categories.ts";

  const props = defineProps<{
    categoryId: number;
  }>();

  const { data: category } = useGetCategory(props.categoryId);
</script>

<template>
  <Badge
    v-if="category"
    variant="outline"
    class="h-6 mx-2 border-none"
    :style="{backgroundColor: `${category.color}25`}"
  >
    <!--	  TODO import from luicide icon -->
    <component :is="category.icon" />
    {{ category.name }}
  </Badge>
</template>

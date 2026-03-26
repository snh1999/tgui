<script setup lang="ts">
  import { computed } from "vue";
  import SelectInput from "@/components/ui/tgui/SelectInput.vue";
  import { useGetCategories } from "@/lib/api/composables/categories.ts";

  const { data: categories } = useGetCategories();
  const shellSelectInputs = computed(() =>
    categories.value?.map((category) => ({
      ...category,
      id: category.id,
      value: category.id,
    }))
  );
</script>

<template>
  <SelectInput :data="shellSelectInputs" placeholder="All categories" />
</template>

<style scoped>
</style>

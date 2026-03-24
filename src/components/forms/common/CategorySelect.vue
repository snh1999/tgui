<script setup lang="ts">
  import { computed } from "vue";
  import SelectInput from "@/components/ui/tgui/SelectInput.vue";
  import { useGetCategories } from "@/lib/api/composables/categories.ts";

  const { data: categories } = useGetCategories();
  const shellSelectInputs = computed(() =>
    categories.value?.map((category) => ({
      id: category.id,
      value: category.id,
      name: category.name,
    }))
  );
</script>

<template>
  <SelectInput :data="shellSelectInputs" />
</template>

<style scoped>
</style>

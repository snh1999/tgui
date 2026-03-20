<script setup lang="ts">
  import { computed } from "vue";
  import SelectInput from "@/components/ui/tgui/SelectInput.vue";
  import { useGetGroups } from "@/lib/api/composables/groups.ts";

  const { data: groups } = useGetGroups();
  const shellSelectInputs = computed(() =>
    groups.value?.map((group) => ({
      id: group.id,
      value: group.id,
      name: group.name,
    }))
  );
</script>

<template>
  <SelectInput :data="shellSelectInputs" />
</template>

<style scoped>
</style>

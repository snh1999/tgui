<script setup lang="ts">
  import { computed } from "vue";
  import SelectInput from "@/components/ui/tgui/SelectInput.vue";
  import { getGetValidShells } from "@/lib/api/composables/process.ts";

  const { data: validShells } = getGetValidShells();
  const shellSelectInputs = computed(() =>
    validShells.value?.map((name) => ({
      id: name,
      value: name,
      name,
    }))
  );
</script>

<template>
  <SelectInput :data="shellSelectInputs" />
</template>

<script setup lang="ts">
  import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
  import { GridIcon, ListIcon2, TableIcon } from "@/assets/Icons.ts";

  const view = defineModel<string>("view", { default: "grid" });
</script>

<template>
  <ToggleGroup v-model="view" type="single">
    <ToggleGroupItem
      value="grid"
      title="Grid View"
      aria-label="Toggle Grid"
      :disabled="view === 'grid'"
    >
      <GridIcon />
    </ToggleGroupItem>
    <ToggleGroupItem
      value="list"
      title="List View"
      aria-label="Toggle List"
      :disabled="view === 'list'"
    >
      <ListIcon2 />
    </ToggleGroupItem>

    <ToggleGroupItem
      value="table"
      title="Table View"
      aria-label="Toggle Table"
      :disabled="view === 'table'"
    >
      <TableIcon />
    </ToggleGroupItem>
  </ToggleGroup>
</template>

<style scoped>
</style>

<script setup lang="ts">
  import CommandCard from "@/components/commands/CommandCard.vue";
  import CommandsTable from "@/components/commands/tables/CommandsTable.vue";
  import DataDisplay from "@/components/views/DataDisplay.vue";
  import { ICommandWithHistory } from "@/lib/api/api.types.ts";
  import { TViewMode } from "@/stores/commands.store.ts";

  defineProps<{
    commands: ICommandWithHistory[];
    view: TViewMode;
    isCard?: boolean;
  }>();
</script>

<template>
  <DataDisplay :view="view">
    <template #list>
      <CommandCard v-for="cmd in commands" :key="cmd.id" :command="cmd" />
    </template>

    <template #grid>
      <CommandCard
        v-for="cmd in commands"
        :key="cmd.id"
        :command="cmd"
        :isCard="isCard ?? true"
      />
    </template>

    <template #table>
      <CommandsTable :commands="commands" />
    </template>
  </DataDisplay>
</template>

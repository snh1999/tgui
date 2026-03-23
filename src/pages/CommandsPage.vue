<script setup lang="ts">
  import CommandCard from "@/components/commands/CommandCard.vue";
  import CommandsTopbar from "@/components/commands/CommandsTopbar.vue";
  import EmptyCommandsPage from "@/components/commands/EmptyCommandsPage.vue";
  import CommandsTable from "@/components/commands/tables/CommandsTable.vue";
  import ErrorDisplay from "@/components/ui/tgui/ErrorDisplay.vue";
  import Loading from "@/components/ui/tgui/Loading.vue";
  import DataDisplay from "@/components/views/DataDisplay.vue";
  import { useGetCommands } from "@/lib/api/composables/commands.ts";
  import { useCommandsViewStore } from "@/stores/commands.store.ts";

  const {
    data: commands,
    isPending,
    isError,
    error,
    refetch,
  } = useGetCommands();

  const commandsView = useCommandsViewStore();
</script>

<template>
  <CommandsTopbar v-if="commands && commands.length > 0" />
  <div class="flex-1 px-6 md:p-8 lg:px-10">
    <ErrorDisplay
      v-if="isError"
      :error="error"
      title="Failed to load commands"
      :retry="refetch"
    />
    <Loading v-else-if="isPending || !commands" />
    <EmptyCommandsPage v-else-if="commands.length === 0" />
    <DataDisplay v-else :view="commandsView.view">
      <template #list>
        <CommandCard v-for="cmd in commands" :key="cmd.id" :command="cmd" />
      </template>

      <template #grid>
        <CommandCard
          v-for="cmd in commands"
          :key="cmd.id"
          :command="cmd"
          isCard
        />
      </template>

      <template #table>
        <CommandsTable :commands="commands" />
      </template>
    </DataDisplay>
  </div>
</template>

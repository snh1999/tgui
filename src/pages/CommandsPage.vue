<script setup lang="ts">
  import CommandsTopbar from "@/components/commands/CommandsTopbar.vue";
  import EmptyCommandsPage from "@/components/commands/EmptyCommandsPage.vue";
  import ErrorDisplay from "@/components/ui/tgui/ErrorDisplay.vue";
  import Loading from "@/components/ui/tgui/Loading.vue";
  import { useGetCommands } from "@/lib/api/composables/commands.ts";
  import { useCommandsStore } from "@/stores/commands.store.ts";
  import CommandsDisplay from "@/components/commands/CommandsDisplay.vue";

  const {
    data: commands,
    isPending,
    isError,
    error,
    refetch,
  } = useGetCommands();

  const commandsView = useCommandsStore();
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
    <CommandsDisplay v-else :commands="commands" :view="commandsView.view" />
  </div>
</template>

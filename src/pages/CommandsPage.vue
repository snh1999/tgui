<script setup lang="ts">
  import CommandsDisplay from "@/components/commands/CommandsDisplay.vue";
  import CommandsTopbar from "@/components/commands/CommandsTopbar.vue";
  import EmptyPage from "@/components/core/EmptyPage.vue";
  import CreateCommandsDialog from "@/components/forms/commands/CreateCommandsDialog.vue";
  import ErrorDisplay from "@/components/ui/tgui/ErrorDisplay.vue";
  import Loading from "@/components/ui/tgui/Loading.vue";
  import { useGetCommands } from "@/lib/api/composables/commands.ts";
  import { useCommandsStore } from "@/stores/commands.store.ts";

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
    <EmptyPage
      v-else-if="commands.length === 0"
      title="No Commands Yet"
      description="You haven't created any commands yet. Get started by creating your first command."
    >
      <div class="flex gap-2">
        <CreateCommandsDialog
          viewTrigger
          triggerSize="lg"
          triggerVariant="primary"
        />
      </div>
    </EmptyPage>
    <CommandsDisplay v-else :commands="commands" :view="commandsView.view" />
  </div>
</template>

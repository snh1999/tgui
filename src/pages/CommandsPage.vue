<script setup lang="ts">
  import { computed } from "vue";
  import CommandsDisplay from "@/components/commands/CommandsDisplay.vue";
  import CommandsTopbar from "@/components/commands/CommandsTopbar.vue";
  import EmptyPage from "@/components/core/EmptyPage.vue";
  import CreateCommandsDialog from "@/components/forms/commands/CreateCommandsDialog.vue";
  import { Button } from "@/components/ui/button";
  import ErrorDisplay from "@/components/ui/tgui/ErrorDisplay.vue";
  import Loading from "@/components/ui/tgui/Loading.vue";
  import { ICommandGroupFilter } from "@/lib/api/api.types.ts";
  import { useGetCommands } from "@/lib/api/composables/commands.ts";
  import { useCommandsStore } from "@/stores/commands.store.ts";
  import { ClearIcon } from "@/assets/Icons.ts";

  const store = useCommandsStore();

  const filters = computed<ICommandGroupFilter>(() => ({
    parentId:
      store.selectedGroup === "none" ? "None" : { Group: store.selectedGroup },
    categoryId:
      store.filterCategory === "all"
        ? "All"
        : { Category: store.filterCategory },
    favoritesOnly: store.showFavoritesOnly,
  }));

  const {
    data: commands,
    isPending,
    isError,
    error,
    refetch,
  } = useGetCommands(filters);

  const commandsView = useCommandsStore();
</script>

<template>
  <CommandsTopbar />
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
      :description="`You haven't created any commands yet. ${store.isFilterChanged?
      'Clear filters to check other commands.' :
      'Get started by creating your first command'}.`"
    >
      <div class="flex gap-2">
        <Button v-if="store.isFilterChanged" @click="store.clearFilter()">
          <ClearIcon />
          Clear Filter
        </Button>
        <CreateCommandsDialog
          v-else
          viewTrigger
          triggerSize="lg"
          triggerVariant="primary"
        />
      </div>
    </EmptyPage>
    <CommandsDisplay v-else :commands="commands" :view="commandsView.view" />
  </div>
</template>

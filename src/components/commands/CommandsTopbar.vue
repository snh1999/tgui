<script setup lang="ts">
  import { computed, ref } from "vue";
  import SearchButton from "@/components/core/titlebar/SearchButton.vue";
  import CreateCommandsDialog from "@/components/forms/commands/CreateCommandsDialog.vue";
  import CategorySelect from "@/components/forms/common/CategorySelect.vue";
  import { Button } from "@/components/ui/button";
  import { Toggle } from "@/components/ui/toggle";
  import DataViewToggle from "@/components/views/DataViewToggle.vue";
  import { useGetCommands } from "@/lib/api/composables/commands.ts";
  import { useCommandsStore } from "@/stores/commands.store.ts";
  import { ClearIcon } from "@/assets/Icons.ts";

  const createDialogOpen = ref(false);

  const { data: commands } = useGetCommands();
  const commandsView = useCommandsStore();

  const filtered = computed(() => commands.value);
  // const filtered = computed(() => {
  //   if(!commands.value) return []
  //   return commands.value.filter((cmd) => {
  //     if (search.value) {
  //       const q = search.value.toLowerCase()
  //       const hit =
  //           cmd.name.toLowerCase().includes(q) ||
  //           cmd.command.toLowerCase().includes(q) ||
  //           cmd.description?.toLowerCase().includes(q) ||
  //           cmd.arguments?.some((a) => a.toLowerCase().includes(q))
  //       if (!hit) return false
  //     }
  //     if (filterCategory.value && cmd.categoryId !== filterCategory.value) return false
  //     if (showFavoritesOnly.value && !cmd.isFavorite) return false
  //     // if (showRunningOnly.value && cmd.status !== 'running') return false
  //     return true
  //   })
  // })
</script>

<template>
  <div class="flex flex-col gap-4 px-5 py-4">
    <div class="flex items-center justify-between gap-3 shrink-0 flex-wrap">
      <div class="flex items-baseline gap-2.5">
        <h1 class="text-base font-bold tracking-[-0.02em]">Commands</h1>
        <span class="text-[11px] text-muted-foreground">
          {{ filtered?.length }} of
          {{ commands?.length }}</span
        >
      </div>
      <div class="flex items-baseline gap-3">
        <SearchButton />
        <CreateCommandsDialog v-model:open="createDialogOpen" view-trigger />
      </div>
    </div>

    <div class="flex items-center gap-2 flex-wrap">
      <Button
        v-if="commandsView.isFilterChanged"
        class="rounded-md bg-destructive/10"
        size="sm"
        @click="commandsView.clearFilter()"
      >
        <ClearIcon />
        Clear Filters
      </Button>

      <div class="flex items-end ml-auto gap-2.5">
        <Toggle v-model="commandsView.showRunningOnly">Running</Toggle>
        <Toggle v-model="commandsView.showFavoritesOnly">
          <span class="h-2 w-2 rounded-full bg-current" />
          Favorites
        </Toggle>
      </div>

      <CategorySelect v-model="commandsView.filterCategory" />
      <DataViewToggle v-model:view="commandsView.view" />
    </div>
  </div>
</template>

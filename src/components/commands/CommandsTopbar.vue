<script setup lang="ts">
  import { computed, ref } from "vue";
  import SearchButton from "@/components/core/titlebar/SearchButton.vue";
  import CreateCommandsDialog from "@/components/forms/commands/CreateCommandsDialog.vue";
  import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from "@/components/ui/select";
  import { Toggle } from "@/components/ui/toggle";
  import DataViewToggle from "@/components/views/DataViewToggle.vue";
  import { useGetCategories } from "@/lib/api/composables/categories.ts";
  import { useGetCommands } from "@/lib/api/composables/commands.ts";
  import { useCommandsStore } from "@/stores/commands.store.ts";

  const createDialogOpen = ref(false);

  const { data: commands } = useGetCommands();
  const { data: categories } = useGetCategories();
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
  <header
    class="flex items-center justify-between px-5 py-4 gap-3 shrink-0 flex-wrap"
  >
    <div class="flex items-baseline gap-2.5">
      <h1 class="text-base font-bold tracking-[-0.02em]">Commands</h1>
      <!--      TODO suspence block-->
      <span class="text-[11px] text-muted-foreground">
        {{ filtered?.length }} of
        {{ commands?.length }}</span
      >
    </div>
    <div class="flex items-center gap-2 flex-wrap">
      <!--      TODO sort by-->

      <div class="flex gap-1">
        <Toggle v-model:pressed="commandsView.showRunningOnly">Running</Toggle>
        <Toggle v-model:pressed="commandsView.showFavoritesOnly">
          <span class="h-2 w-2 rounded-full bg-current" />
          Favorites
        </Toggle>
      </div>

      <DataViewToggle v-model:view="commandsView.view" />

      <CreateCommandsDialog v-model:open="createDialogOpen" view-trigger />
    </div>
    <div class="flex items-end ml-auto gap-2.5">
      <SearchButton />
      <Select v-model="commandsView.filterCategory">
        <SelectTrigger class="w-30 md:w-35">
          <SelectValue placeholder="All categories" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="all">All categories</SelectItem>
          <SelectItem v-for="c in categories" :key="c.id" :value="c.id">
            <span class="mr-2">{{ c.icon }}</span>
            {{ c.name }}
          </SelectItem>
        </SelectContent>
      </Select>
    </div>
  </header>
</template>

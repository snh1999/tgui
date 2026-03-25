<script setup lang="ts">
  import { computed } from "vue";
  import { useRoute } from "vue-router";
  import CategoryTopbar from "@/components/categories/CategoryTopbar.vue";
  import CategoryFullScreenView from "@/components/categories/views/CategoryFullScreenView.vue";
  import CategorySplitView from "@/components/categories/views/CategorySplitView.vue";
  import ErrorDisplay from "@/components/ui/tgui/ErrorDisplay.vue";
  import Loading from "@/components/ui/tgui/Loading.vue";
  import type { ICommandGroupFilter } from "@/lib/api/api.types.ts";
  import { useGetCategory } from "@/lib/api/composables/categories.ts";
  import { useGetCommands } from "@/lib/api/composables/commands.ts";
  import { useGetGroups } from "@/lib/api/composables/groups.ts";
  import { useAppStore } from "@/stores/app.store.ts";

  const route = useRoute();
  const categoryId = computed(() => Number(route.params.id));
  const appStore = useAppStore();

  const {
    data: category,
    isPending: categoryLoading,
    isError: categoryError,
    error: categoryErr,
    refetch: refetchCategory,
  } = useGetCategory(categoryId);

  const filters = computed<ICommandGroupFilter>(() => ({
    parentId: "All",
    categoryId: { Category: categoryId.value },
    favoritesOnly: false,
  }));

  const {
    data: commands,
    isPending: commandsLoading,
    isError: commandsError,
    error: commandsErr,
    refetch: refetchCommands,
  } = useGetCommands(filters);

  const {
    data: groups,
    isPending: groupsLoading,
    isError: groupsError,
    error: groupsErr,
    refetch: refetchGroups,
  } = useGetGroups(filters);

  const isLoading = computed(
    () => categoryLoading.value || commandsLoading.value || groupsLoading.value
  );
  const hasError = computed(
    () => categoryError.value || commandsError.value || groupsError.value
  );
  const anyError = computed(
    () => categoryErr.value || commandsErr.value || groupsErr.value
  );
  const anyRefetch = computed(
    () => refetchCategory || refetchCommands || refetchGroups
  );
</script>

<template>
  <div class="flex flex-col h-full w-full overflow-hidden">
    <ErrorDisplay
      v-if="hasError"
      :error="anyError"
      title="Failed to load data"
      :retry="anyRefetch"
    />
    <Loading v-else-if="isLoading" />

    <template v-else-if="category">
      <CategoryTopbar :category="category" />

      <CategoryFullScreenView
        v-if="appStore.layoutState === 'full screen'"
        :commands="commands"
        :groups="groups"
      />

      <CategorySplitView
        v-else
        :layout="appStore.layoutState"
        :commands="commands"
        :groups="groups"
      />
    </template>
  </div>
</template>

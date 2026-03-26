<script setup lang="ts">
  import { computed } from "vue";
  import { useRoute } from "vue-router";
  import CategoryTopbar from "@/components/categories/CategoryTopbar.vue";
  import ErrorDisplay from "@/components/ui/tgui/ErrorDisplay.vue";
  import Loading from "@/components/ui/tgui/Loading.vue";
  import { useGetGroupCommand } from "@/components/views/group-command/GroupCommand.helpers.ts";
  import GroupCommandFullScreenView from "@/components/views/group-command/GroupCommandFullScreenView.vue";
  import GroupCommandSplitView from "@/components/views/group-command/GroupCommandSplitView.vue";
  import type { ICommandGroupFilter } from "@/lib/api/api.types.ts";
  import { useGetCategory } from "@/lib/api/composables/categories.ts";
  import { useAppStore } from "@/stores/app.store.ts";

  const route = useRoute();
  const categoryId = computed(() => Number(route.params.id));
  const appStore = useAppStore();

  const {
    data: category,
    isLoading: categoryLoading,
    isError: categoryError,
    error: categoryErr,
    refetch: refetchCategory,
  } = useGetCategory(categoryId);

  const filters = computed<ICommandGroupFilter>(() => ({
    parentId: "All",
    categoryId: { Category: categoryId.value },
    favoritesOnly: false,
  }));

  const { groups, commands, isValuesLoading, isError, error, refetch } =
    useGetGroupCommand(filters);

  const isLoading = computed(
    () => categoryLoading.value || isValuesLoading.value
  );
  const hasError = computed(() => categoryError.value || isError.value);
  const anyError = computed(() => categoryErr.value || error.value);
  const anyRefetch = computed(() => refetchCategory || refetch);
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

      <GroupCommandFullScreenView
        v-if="appStore.layoutState === 'full screen'"
        :commands="commands"
        :groups="groups"
      />

      <GroupCommandSplitView
        v-else
        :layout="appStore.layoutState"
        :commands="commands"
        :groups="groups"
      />
    </template>
  </div>
</template>

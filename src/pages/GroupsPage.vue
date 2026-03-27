<script setup lang="ts">
  import { computed } from "vue";
  import { useRoute } from "vue-router";
  import GroupsTopbar from "@/components/groups/GroupsTopbar.vue";
  import ErrorDisplay from "@/components/ui/tgui/ErrorDisplay.vue";
  import Loading from "@/components/ui/tgui/Loading.vue";
  import { useGetGroupCommand } from "@/components/views/group-command/GroupCommand.helpers.ts";
  import GroupCommandFullScreenView from "@/components/views/group-command/GroupCommandFullScreenView.vue";
  import GroupCommandSplitView from "@/components/views/group-command/GroupCommandSplitView.vue";
  import type { ICommandGroupFilter } from "@/lib/api/api.types.ts";
  import { useGetGroup } from "@/lib/api/composables/groups.ts";
  import { useAppStore } from "@/stores/app.store.ts";
  import { useGroupsStore } from "@/stores/groups.store.ts";

  const route = useRoute();
  const groupId = computed(() => Number(route.params.id));
  const appStore = useAppStore();
  const groupsStore = useGroupsStore();

  const {
    data: group,
    isLoading: groupLoading,
    isError: groupIsError,
    error: groupError,
    refetch: groupRefetch,
  } = useGetGroup(groupId);
  const filters = computed<ICommandGroupFilter>(() => ({
    parentId: Number.isNaN(groupId.value) ? "None" : { Group: groupId.value },
    categoryId:
      groupsStore.filterCategory === "all"
        ? "All"
        : { Category: groupsStore.filterCategory },
    favoritesOnly: groupsStore.favoritesOnly,
  }));

  const { groups, commands, isValuesLoading, isError, error, refetch } =
    useGetGroupCommand(filters);

  const isLoading = computed(() => groupLoading.value || isValuesLoading.value);
  const hasError = computed(() => groupIsError.value || isError.value);
  const anyError = computed(() => groupError.value || error.value);
  const anyRefetch = computed(() => groupRefetch || refetch);
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

    <template v-else>
      <GroupsTopbar :group="group" />

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

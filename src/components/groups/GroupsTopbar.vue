<script setup lang="ts">
  import { Copy, X } from "lucide-vue-next";
  import { ref } from "vue";
  import { useRouter } from "vue-router";
  import { DeleteIcon } from "@/assets/Icons.ts";
  import CategorySelect from "@/components/forms/common/CategorySelect.vue";
  import CreateGroupDialog from "@/components/forms/groups/CreateGroupDialog.vue";
  import UpdateGroupDialog from "@/components/forms/groups/UpdateGroupDialog.vue";
  import GroupCategoryLine from "@/components/shared/GroupCategoryLine.vue";
  import { Button } from "@/components/ui/button";

  import ConfirmDialog from "@/components/ui/tgui/ConfirmDialog.vue";
  import { Toggle } from "@/components/ui/toggle";
  import { IGroup } from "@/lib/api/api.types.ts";
  import { useDeleteGroup } from "@/lib/api/composables/groups.ts";
  import { useGroupsStore } from "@/stores/groups.store.ts";

  const props = defineProps<{
    group?: IGroup;
  }>();

  const groupsView = useGroupsStore();

  const deleteDialogOpen = ref(false);

  const { mutate: deleteGroup } = useDeleteGroup();

  const emit = defineEmits<{
    delete: [id: number];
  }>();

  const router = useRouter();

  function onDeleteClick() {
    if (!props.group) {
      return;
    }
    deleteGroup(props.group.id);
    deleteDialogOpen.value = false;
    emit("delete", props.group.id);
    router.back();
  }
</script>

<template>
  <div class="px-5 py-4 flex flex-col gap-4">
    <div class="flex items-center justify-between  gap-3 shrink-0 flex-wrap">
      <div class="flex items-baseline gap-2.5">
        <h1 v-if="!group" class="text-base font-bold tracking-[-0.02em]">
          Groups
        </h1>
        <GroupCategoryLine v-else :element="group" />
      </div>

      <div v-if="group" class="flex items-center gap-2">
        <UpdateGroupDialog
          :id="group.id"
          triggerVariant="outline"
          viewTrigger
        />
        <CreateGroupDialog :group="group">
          <Copy class="h-3.5 w-3.5" />
          Duplicate
        </CreateGroupDialog>

        <Button
          variant="destructive"
          size="sm"
          title="Delete Group"
          @click="deleteDialogOpen = true"
        >
          <DeleteIcon class="size-4" />
          <span class="ml-1">Delete</span>
        </Button>
      </div>

      <ConfirmDialog
        v-model:open="deleteDialogOpen"
        description="Are you sure you want to delete this Group? Commands and child groups will be updated to null."
        @confirm="onDeleteClick"
      />
    </div>
    <div
      class="flex"
      :class="groupsView.isFilterChanged? 'justify-between': 'justify-end'"
    >
      <Button
        v-if="groupsView.isFilterChanged"
        class="rounded-md bg-destructive/10"
        size="sm"
        @click="groupsView.clearFilter()"
      >
        Clear Filters <X />
      </Button>
      <div class="flex gap-3">
        <CategorySelect v-model="groupsView.filterCategory" />

        <Toggle v-model="groupsView.favoritesOnly">
          <span class="h-2 w-2 rounded-full bg-current" />
          Favorites
        </Toggle>
      </div>
    </div>
  </div>
</template>

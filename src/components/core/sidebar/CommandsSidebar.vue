<script setup lang="ts">
  import { ChevronLeft } from "lucide-vue-next";
  import { computed, provide } from "vue";
  import { useRouter } from "vue-router";
  import { AddIcon, SelectAllIcon } from "@/assets/Icons.ts";
  import GroupTreeNode from "@/components/core/sidebar/groups/GroupTreeNode.vue";
  import CreateGroupDialog from "@/components/forms/groups/CreateGroupDialog.vue";
  import { Button } from "@/components/ui/button";
  import {
    SidebarContent,
    SidebarHeader,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
  } from "@/components/ui/sidebar";
  import Loading from "@/components/ui/tgui/Loading.vue";
  import { useGetGroups } from "@/lib/api/composables/groups.ts";
  import { useCommandsStore } from "@/stores/commands.store.ts";

  const router = useRouter();
  const store = useCommandsStore();

  const { data: rootGroups, isLoading } = useGetGroups({
    parentId: "None",
    categoryId: "All",
    favoritesOnly: false,
  });

  provide("onGroupSelected", (id: number) => {
    store.selectedGroup = id;
  });

  provide(
    "selectedGroupId",
    computed(() => store.selectedGroup)
  );
</script>

<template>
  <SidebarHeader>
    <SidebarMenu>
      <div class="flex items-center justify-between">
        <Button
          @click="router.back()"
          class="w-max border-none"
          variant="ghost"
        >
          <ChevronLeft class="w-4 h-4" />
        </Button>
        <CreateGroupDialog triggerVariant="ghost" triggerSize="sm">
          <AddIcon />
        </CreateGroupDialog>
      </div>
    </SidebarMenu>
  </SidebarHeader>

  <SidebarContent class="px-1">
    <SidebarMenu>
      <SidebarMenuItem>
        <SidebarMenuButton
          @click="store.selectedGroup='none'"
          :isActive="store.selectedGroup === 'none'"
        >
          <SelectAllIcon />
          Root Groups
        </SidebarMenuButton>
      </SidebarMenuItem>
      <Loading v-if="isLoading" />
      <GroupTreeNode
        v-else
        v-for="group in rootGroups"
        :key="group.id"
        :group="group"
        showLinkButton
      />
    </SidebarMenu>
  </SidebarContent>
</template>

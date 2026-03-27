<script setup lang="ts">
  import { ChevronLeft, ChevronRight } from "lucide-vue-next";
  import { ComputedRef, computed } from "vue";
  import { useRoute, useRouter } from "vue-router";
  import { AddIcon } from "@/assets/Icons.ts";
  import CreateGroupDialog from "@/components/forms/groups/CreateGroupDialog.vue";
  import GroupCategoryLine from "@/components/shared/GroupCategoryLine.vue";
  import { Button } from "@/components/ui/button";
  import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
  } from "@/components/ui/collapsible";
  import {
    SidebarHeader,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
    SidebarMenuSub,
  } from "@/components/ui/sidebar";
  import ErrorDisplay from "@/components/ui/tgui/ErrorDisplay.vue";
  import Loading from "@/components/ui/tgui/Loading.vue";
  import { ICommandGroupFilter } from "@/lib/api/api.types.ts";
  import { useGetGroup, useGetGroups } from "@/lib/api/composables/groups.ts";
  import { routePaths } from "@/router";

  const route = useRoute();
  const router = useRouter();

  const groupId = computed(() => Number(route.params.id));
  const { data: group } = useGetGroup(groupId);

  const filter: ComputedRef<ICommandGroupFilter> = computed(() => ({
    parentId: Number.isNaN(groupId.value) ? "None" : { Group: groupId.value },
    categoryId: "All",
    favoritesOnly: false,
  }));

  const {
    data: groups,
    isLoading,
    error,
    isError,
    refetch,
  } = useGetGroups(filter);
</script>

<template>
  <Loading v-if="isLoading" />
  <ErrorDisplay v-if="isError" :error="error" :retry="refetch" />

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

  <SidebarMenuItem>
    <Collapsible default-open class="group/collapsible">
      <CollapsibleTrigger as-child>
        <SidebarMenuButton>
          <GroupCategoryLine v-if="group" :element="group" />
          <span v-else>Root Groups</span>
          <ChevronRight
            class="ml-auto transition-transform group-data-[state=open]/collapsible:rotate-90"
          />
        </SidebarMenuButton>
      </CollapsibleTrigger>
      <CollapsibleContent>
        <SidebarMenuSub>
          <SidebarMenuItem v-for="group in groups" :key="group.id" class="px-2">
            <SidebarMenuButton
              @click="router.push(`${routePaths.groups}/${group.id}`)"
              :isActive="group.id === groupId"
            >
              <GroupCategoryLine :element="group" />
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenuSub>
      </CollapsibleContent>
    </Collapsible>
  </SidebarMenuItem>
</template>

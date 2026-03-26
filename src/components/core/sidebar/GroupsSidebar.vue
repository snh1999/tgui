<script setup lang="ts">
  import { ChevronLeft } from "lucide-vue-next";
  import { ComputedRef, computed } from "vue";
  import { useRoute, useRouter } from "vue-router";
  import GroupCategoryLine from "@/components/shared/GroupCategoryLine.vue";
  import { Button } from "@/components/ui/button";
  import {
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
  } from "@/components/ui/sidebar";
  import ErrorDisplay from "@/components/ui/tgui/ErrorDisplay.vue";
  import Loading from "@/components/ui/tgui/Loading.vue";
  import { ICommandGroupFilter } from "@/lib/api/api.types.ts";
  import { useGetGroups } from "@/lib/api/composables/groups.ts";
  import { routePaths } from "@/router";

  const route = useRoute();
  const router = useRouter();

  const id = computed(() => Number(route.params.id));

  const filter: ComputedRef<ICommandGroupFilter> = computed(() => ({
    parentId: Number.isNaN(id.value) ? "None" : { Group: id.value },
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

  <SidebarMenu>
    <Button @click="router.back()" class="w-max border-none" variant="ghost">
      <ChevronLeft class="w-4 h-4" />
    </Button>
  </SidebarMenu>

  <SidebarMenu>
    <SidebarMenuItem v-for="group in groups" :key="group.id" class="px-2">
      <SidebarMenuButton
        @click="router.push(`${routePaths.categories}/${group.id}`)"
        :isActive="group.id === id"
      >
        <GroupCategoryLine :element="group" />
      </SidebarMenuButton>
    </SidebarMenuItem>
  </SidebarMenu>
</template>

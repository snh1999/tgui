<script setup lang="ts">
  import { ChevronLeft } from "lucide-vue-next";
  import { useRoute, useRouter } from "vue-router";
  import { AddIcon } from "@/assets/Icons.ts";
  import CreateCategoryDialog from "@/components/forms/categories/CreateCategoryDialog.vue";
  import GroupCategoryLine from "@/components/shared/GroupCategoryLine.vue";
  import { Button } from "@/components/ui/button";
  import {
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
  } from "@/components/ui/sidebar";
  import ErrorDisplay from "@/components/ui/tgui/ErrorDisplay.vue";
  import Loading from "@/components/ui/tgui/Loading.vue";
  import { useGetCategories } from "@/lib/api/composables/categories.ts";
  import { routePaths } from "@/router";

  const router = useRouter();
  const route = useRoute();

  const {
    data: categories,
    isLoading,
    error,
    isError,
    refetch,
  } = useGetCategories();
</script>

<template>
  <Loading v-if="isLoading" />
  <ErrorDisplay v-if="isError" :error="error" :retry="refetch" />

  <SidebarMenu>
    <SidebarMenu class="p-2">
      <div class="flex items-center justify-between">
        <Button
          @click="router.back()"
          class="w-max border-none"
          variant="ghost"
        >
          <ChevronLeft class="w-4 h-4" />
        </Button>

        <CreateCategoryDialog triggerVariant="ghost" triggerSize="sm">
          <AddIcon />
        </CreateCategoryDialog>
      </div>
    </SidebarMenu>
  </SidebarMenu>

  <SidebarMenu>
    <SidebarMenuItem
      v-for="category in categories"
      :key="category.id"
      class="px-2"
    >
      <SidebarMenuButton
        @click="router.push(`${routePaths.categories}/${category.id}`)"
        :isActive="category.id === Number(route.params.id)"
      >
        <GroupCategoryLine :element="category" />
      </SidebarMenuButton>
    </SidebarMenuItem>
  </SidebarMenu>
</template>

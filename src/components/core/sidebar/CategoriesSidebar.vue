<script setup lang="ts">
  import { useGetCategories } from "@/lib/api/composables/categories.ts";
  import {
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
  } from "@/components/ui/sidebar";
  import { Button } from "@/components/ui/button";
  import { ChevronLeft, HomeIcon, RotateCcw } from "lucide-vue-next";
  import { useRoute, useRouter } from "vue-router";
  import Loading from "@/components/ui/tgui/Loading.vue";
  import ErrorDisplay from "@/components/ui/tgui/ErrorDisplay.vue";
  import { routePaths } from "@/router";
  import CategoryLine from "@/components/categories/CategoryLine.vue";

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

  <SidebarMenu class="sticky top-0">
    <div class="flex items-center justify-between">
      <div class="sticky top-0 flex py-2">
        <Button
          @click="router.back()"
          class="w-max border-none"
          variant="ghost"
        >
          <ChevronLeft class="w-4 h-4" />
        </Button>
        <Button
          @click="router.push('/')"
          class="w-max border-none"
          variant="ghost"
        >
          <HomeIcon class="w-4 h-4" />
        </Button>
      </div>
      <Button @click="router.go(0)" variant="ghost" class="w-max border-none">
        <RotateCcw />
      </Button>
    </div>
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
        <CategoryLine :category="category" />
      </SidebarMenuButton>
    </SidebarMenuItem>
  </SidebarMenu>
</template>

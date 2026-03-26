<script setup lang="ts">
  import { type Component, computed } from "vue";
  import { useRoute } from "vue-router";
  import CategoriesSidebar from "@/components/core/sidebar/CategoriesSidebar.vue";
  import CommandsSidebar from "@/components/core/sidebar/CommandsSidebar.vue";
  import GroupsSidebar from "@/components/core/sidebar/GroupsSidebar.vue";
  import HomeSidebar from "@/components/core/sidebar/HomeSidebar.vue";
  import SettingsSidebar from "@/components/core/sidebar/SettingsSidebar.vue";
  import { Sidebar, SidebarContent } from "@/components/ui/sidebar";
  import { TRoutePaths } from "@/router";

  const route = useRoute();

  const routeComponents: Record<string, Component> = {
    home: HomeSidebar,
    groups: GroupsSidebar,
    group: GroupsSidebar,
    commands: CommandsSidebar,
    command: CommandsSidebar,
    categories: CategoriesSidebar,
    category: CategoriesSidebar,
    settings: SettingsSidebar,
  };

  const SidebarComponent = computed(() => {
    const name = route.name as TRoutePaths;
    return routeComponents[name] || HomeSidebar;
  });
</script>

<template>
  <Sidebar collapsible="icon" side="left" variant="sidebar" :top-offset="40">
    <SidebarContent>
      <Component :is="SidebarComponent" />
    </SidebarContent>
  </Sidebar>
</template>

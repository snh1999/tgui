<script setup lang="ts">
  import { ListPlus, Settings } from "lucide-vue-next";
  import { useRoute } from "vue-router";
  import { CategoryIcon, GroupIcon } from "@/assets/Icons.ts";
  import {
    SidebarGroup,
    SidebarGroupContent,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
  } from "@/components/ui/sidebar";

  const route = useRoute();
  const isMenuActive = (path: string) => path === route.path;

  const items = [
    {
      title: "Commands",
      url: "/commands",
      icon: ListPlus,
    },
    {
      title: "Groups",
      url: "/groups",
      icon: GroupIcon,
    },
    {
      title: "Categories",
      url: "/categories",
      icon: CategoryIcon,
    },
    {
      title: "Settings",
      url: "/settings",
      icon: Settings,
    },
  ];
</script>

<template>
  <SidebarGroup>
    <SidebarGroupContent>
      <SidebarMenu>
        <SidebarMenuItem v-for="item in items" :key="item.title">
          <SidebarMenuButton as-child :is-active="isMenuActive(item.url)">
            <RouterLink :to="item.url">
              <component :is="item.icon" />
              <span>{{ item.title }}</span>
            </RouterLink>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarGroupContent>
  </SidebarGroup>
</template>

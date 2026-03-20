<script setup lang="ts">
  import { ChevronRight, ListPlus, Plus, Settings } from "lucide-vue-next";
  import { useRoute } from "vue-router";
  import { DashboardIcon, GroupIcon } from "@/assets/Icons.ts";
  import logo from "@/assets/logo.svg";
  import CreateCategoryDialog from "@/components/forms/categories/CreateCategoryDialog.vue";
  import TitleBar from "@/components/core/title-bar/TitleBar.vue";
  import { Collapsible } from "@/components/ui/collapsible";
  import {
    Sidebar,
    SidebarContent,
    SidebarGroup,
    SidebarGroupAction,
    SidebarGroupContent,
    SidebarGroupLabel,
    SidebarHeader,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
    SidebarTrigger,
    useSidebar,
  } from "@/components/ui/sidebar";
  import { useGetCategories } from "@/lib/api/composables/categories.ts";
  import { appStateStore } from "@/stores/app.store.ts";

  const sidebarState = useSidebar();
  const route = useRoute();
  const isMenuActive = (path: string) => path === route.path;

  const items = [
    {
      title: "Dashboard",
      url: "/",
      icon: DashboardIcon,
    },
    {
      title: "Commands",
      url: "/commands",
      icon: ListPlus,
    },
    {
      title: "Browse",
      url: "/browse",
      icon: ListPlus,
    },
    {
      title: "Groups",
      url: "/groups",
      icon: GroupIcon,
    },
    {
      title: "Settings",
      url: "/settings",
      icon: Settings,
    },
  ];

  const { data: categories } = useGetCategories();
</script>

<template>
  <Sidebar collapsible="icon" side="left" variant="sidebar" :top-offset="40">
    <SidebarHeader>
      <div
        v-show="sidebarState.open.value"
        class="mb-3 mt-2 flex items-center justify-center gap-3"
      >
        <img class="h-8 w-8" alt="logo" :src="logo">
        <div class="text-xl font-semibold tracking-tighter">TGUI</div>
      </div>
    </SidebarHeader>
    <SidebarContent>
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
      <SidebarGroup as-child>
        <Collapsible default-open class="group/collapsible">
          <SidebarGroupLabel as-child>
            <CollapsibleTrigger
              class="group/label w-full text-left text-sm text-sidebar-foreground hover:bg-sidebar-accent/70 hover:text-sidebar-accent-foreground [&[data-state=open]>svg]:rotate-90"
            >
              Categories
              <ChevronRight
                class="transition-transform group-data-[state=open]/collapsible:rotate-90"
              />
            </CollapsibleTrigger>
            <SidebarGroupAction>
              <Plus />
              <CreateCategoryDialog viewTrigger />
            </SidebarGroupAction>
          </SidebarGroupLabel>
          <CollapsibleContent>
            <SidebarGroupContent>
              <SidebarMenu>
                <SidebarMenuItem
                  v-for="category in categories"
                  :key="category.id"
                >
                  {{ category.name }}
                </SidebarMenuItem>
              </SidebarMenu>
            </SidebarGroupContent>
          </CollapsibleContent>
        </Collapsible>
      </SidebarGroup>
    </SidebarContent>
  </Sidebar>
</template>

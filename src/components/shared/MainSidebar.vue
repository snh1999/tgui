<script setup lang="ts">
import { ChevronRight, ListPlus, Plus, Settings } from "lucide-vue-next";
import { DashboardIcon } from "@/assets/Icons.ts";

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
import logo from "@/assets/logo.svg";
import { Collapsible } from "@/components/ui/collapsible";
import { useRoute } from "vue-router";

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
        title: "Settings",
        url: "/settings",
        icon: Settings,
    },
];
</script>

<template>
    <Sidebar collapsible="icon" side="left" variant="sidebar">
        <SidebarHeader>
            <div class="grid grid-cols-[1fr_auto_1fr] mb-3 mt-2">
                <div />
                <div
                    v-show="sidebarState.open.value"
                    class="flex items-center justify-center gap-3"
                >
                    <img class="h-8 w-8" alt="logo" :src="logo" />
                    <div class="text-xl font-semibold tracking-tighter">
                        TGUI
                    </div>
                </div>
                <div class="flex justify-end">
                    <SidebarTrigger />
                </div>
            </div>
        </SidebarHeader>
        <SidebarContent>
            <SidebarGroup>
                <SidebarGroupContent>
                    <SidebarMenu>
                        <SidebarMenuItem
                            v-for="item in items"
                            :key="item.title"
                        >
                            <SidebarMenuButton
                                as-child
                                :is-active="isMenuActive(item.url)"
                            >
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
                            class="group/label w-full text-left text-sm text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground [&[data-state=open]>svg]:rotate-90"
                        >
                            Group
                            <ChevronRight
                                class="transition-transform group-data-[state=open]/collapsible:rotate-90"
                            />
                        </CollapsibleTrigger>
                        <SidebarGroupAction>
                            <Plus />
                        </SidebarGroupAction>
                    </SidebarGroupLabel>
                    <CollapsibleContent>
                        <SidebarGroupContent>
                            <SidebarMenu>
                                <!-- Menu items -->
                            </SidebarMenu>
                        </SidebarGroupContent>
                    </CollapsibleContent>
                </Collapsible>
            </SidebarGroup>
        </SidebarContent>
    </Sidebar>
</template>

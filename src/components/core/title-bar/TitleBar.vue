<script setup lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import {  X } from "lucide-vue-next";
  import { onMounted, ref } from "vue";
  import {
    MaximizeIcon,
    MinimizeIcon,
    UnMaximizeIcon,
  } from "@/assets/Icons.ts";
  import { SidebarTrigger } from "@/components/ui/sidebar";
  import { Button } from "@/components/ui/button";
  import ActionButtons from "@/components/core/title-bar/ActionButtons.vue";
  import ThemeSwitcher from "@/components/core/ThemeSwitcher.vue";

  // const route = useRoute();
  const appWindow = getCurrentWindow();
  const isMaximized = ref(false);

  // const breadcrumb = computed(() => {
  //   const name = route.name as string;
  //   const labels: Record<string, string> = {
  //     dashboard: "Dashboard",
  //     groups: "Groups",
  //     commands: "Commands",
  //     "command-detail": "Command",
  //     browse: "Browse",
  //     settings: "Settings",
  //   };
  //   return {
  //     app: "TGUI",
  //     current: labels[name] || name || "Root",
  //   };
  // });

  onMounted(async () => {
    isMaximized.value = await appWindow.isMaximized();
    await appWindow.onResized(async () => {
      isMaximized.value = await appWindow.isMaximized();
    });
  });

  const minimize = () => appWindow.minimize();
  const toggleMaximize = () => appWindow.toggleMaximize();
  const close = () => appWindow.close();
  const startDrag = () => appWindow.startDragging();
</script>

<template>
  <!--TODO: make cursor grabbing while dragging-->
  <header
    class="flex items-center justify-between h-10 pl-2 text-card-foreground bg-card border-b select-none shrink-0"
    @mousedown="startDrag"
  >
    <div class="flex items-center h-full" @mousedown.stop>
      <SidebarTrigger />
      <ActionButtons />
      <!--      <div class="flex items-center h-full px-2 text-sm">-->
      <!--        <span class="text-muted-foreground">{{ breadcrumb.app }}</span>-->
      <!--        <ChevronRight class="w-3 h-3 mx-1 text-muted-foreground" />-->
      <!--        <span class="font-medium">{{ breadcrumb.current }}</span>-->
      <!--      </div>-->
    </div>
    <div class="flex items-center h-full" @mousedown.stop>
      <ThemeSwitcher />
      <Button size="icon" variant="ghost" @click="minimize">
        <MinimizeIcon class="scale-105" />
      </Button>
      <Button
        size="icon-lg"
        variant="ghost"
        class="h-full"
        @click="toggleMaximize"
      >
        <UnMaximizeIcon v-if="isMaximized" class="scale-85" />
        <MaximizeIcon v-else class="scale-85" />
      </Button>
      <Button
        size="icon-lg"
        variant="ghost"
        class="h-full hover:bg-destructive hover:text-destructive-foreground"
        @click="close"
      >
        <X />
      </Button>
    </div>
  </header>
</template>

<script setup lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { Columns2, RotateCcw, Rows2, Square, X } from "lucide-vue-next";
  import { onMounted, ref } from "vue";
  import { useRouter } from "vue-router";
  import {
    MaximizeIcon,
    MinimizeIcon,
    UnMaximizeIcon,
  } from "@/assets/Icons.ts";
  import ThemeSwitcher from "@/components/core/ThemeSwitcher.vue";
  import ActionButtons from "@/components/core/titlebar/ActionButtons.vue";
  import TitleBreadcrumb from "@/components/core/titlebar/TitleBreadcrumb.vue";
  import { Button } from "@/components/ui/button";
  import { SidebarTrigger } from "@/components/ui/sidebar";
  import { useAppStore } from "@/stores/app.store.ts";

  const appWindow = getCurrentWindow();
  const isMaximized = ref(false);

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

  const router = useRouter();

  const appStore = useAppStore();
</script>

<template>
  <!--TODO: make cursor grabbing while dragging-->
  <header
    class="flex items-center justify-between h-10 pl-2 text-card-foreground bg-card border-b select-none shrink-0"
    @mousedown="startDrag"
  >
    <div class="flex items-center h-full" @mousedown.stop>
      <SidebarTrigger />
      <Button @click="router.go(0)" variant="ghost" class="w-max border-none">
        <RotateCcw />
      </Button>
      <ActionButtons />
      <TitleBreadcrumb />
    </div>
    <div class="flex items-center h-full" @mousedown.stop>
      <ThemeSwitcher />
      <Button variant="ghost" @click="appStore.toggleLayoutState()">
        <Columns2 v-if="appStore.layoutState === 'horizontal'" />
        <Rows2 v-else-if="appStore.layoutState === 'vertical'" />
        <Square v-else />
      </Button>
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
      <Button size="icon-lg" variant="destructive_hover" @click="close">
        <X />
      </Button>
    </div>
  </header>
</template>

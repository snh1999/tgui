<script setup lang="ts">
  import MainSidebar from "@/components/shared/MainSidebar.vue";
  import { SidebarProvider } from "@/components/ui/sidebar";
</script>

<template>
  <SidebarProvider storage-key="sidebar">
    <MainSidebar />
    <main
      class="flex-1 flex flex-col h-full overflow-auto bg-background text-foreground"
    >
      <slot />
      <RouterView />
    </main>
  </SidebarProvider>
</template>

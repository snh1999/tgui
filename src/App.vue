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

<style scoped>
  .app-container {
    display: flex;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    background: var(--color-bg);
  }

  .content-scroll {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: var(--space-lg);
  }

  /* At 1/3 screen (~400-500px), collapse sidebar */
  @container root (max-width: 500px) {
    :root {
      --sidebar-width: var(--sidebar-collapsed-width);
    }
  }
</style>

<script setup lang="ts">
  import LogViewer from "@/components/logs/LogViewer.vue";
  import StatsSection from "@/components/stats/StatsSection.vue";
  import Topbar from "@/components/Topbar.vue";
  import TableWrapper from "@/components/table/TableWrapper.vue";
  import Sidebar from "./components/Sidebar.vue";
</script>

<template>
  <div class="app-container">
    <Sidebar />
    <main class="main-content">
      <Topbar />
      <div class="content-scroll">
        <StatsSection />
        <TableWrapper />
        <LogViewer />
      </div>
    </main>
  </div>
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

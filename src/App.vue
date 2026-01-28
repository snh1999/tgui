<script setup lang="ts">
import LogViewer from "@/components/logs/LogViewer.vue";
import StatsSection from "@/components/stats/StatsSection.vue";
import Topbar from "@/components/Topbar.vue";
import TableWrapper from "@/components/table/TableWrapper.vue";
import Sidebar from "./components/Sidebar.vue";
</script>

<template>
  <body>
  <div class="container">
    <Sidebar/>
    <main class="main-content">
      <Topbar/>

      <StatsSection/>
      <TableWrapper/>
      <LogViewer/>
    </main>
    x
  </div>
  </body>
</template>

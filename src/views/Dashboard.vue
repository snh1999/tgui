<script setup lang="ts">
import LogViewer from "@/components/logs/LogViewer.vue";
import Topbar from "@/components/shared/Topbar.vue";
import StatsSection from "@/components/stats/StatsSection.vue";
import CommandsTable from "@/components/tables/commands/CommandsTable.vue";
</script>

<template>
    <main>
        <Topbar />
        <div class="flex-1 px-6 md:p-8 lg:px-10">
            <StatsSection />
            <!--TODO add favorites section-->
            <!--TODO add recent section-->
            <CommandsTable />
            <LogViewer />
        </div>
    </main>
</template>

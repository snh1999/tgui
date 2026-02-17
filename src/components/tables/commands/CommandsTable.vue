<script setup lang="ts">
  import { columns } from "@/components/tables/commands/columns.ts";
  import { Badge } from "@/components/ui/badge";
  import DataTable from "@/components/ui/table/DataTable.vue";
  import ErrorDisplay from "@/components/ui/tgui/ErrorDisplay.vue";
  import Loading from "@/components/ui/tgui/Loading.vue";
  import { useGetCommands } from "@/lib/api/composables/commands.ts";

  const {
    data: commands,
    isPending,
    isError,
    error,
    refetch,
  } = useGetCommands();
</script>

<template>
  <section
    class="flex flex-col rounded-md border border-muted bg-card text-card-foreground mt-5"
  >
    <Loading v-if="isPending" />
    <div
      class="flex items-center justify-between px-6 py-4 border-b border-b-muted"
    >
      <div class="flex items-center gap-3">
        <h2 class="text-lg font-semibold tracking-tight">Command Registry</h2>
        <Badge>8 active</Badge>
      </div>
    </div>
    <div class="relative w-full overflow-auto">
      <ErrorDisplay
        v-if="isError"
        :error="error"
        title="Failed to load commands"
        :retry="refetch"
      />
      <DataTable
        :data="commands ?? []"
        :columns="columns"
        footer-content="test"
      />
    </div>
  </section>
</template>

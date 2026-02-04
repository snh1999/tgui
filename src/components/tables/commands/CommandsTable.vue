<script setup lang="ts">
  import { columns, data } from "./columns";
  import DataTable from "@/components/ui/table/DataTable.vue";
  import { Badge } from "@/components/ui/badge";

  // async function getData(): Promise<Command[]> {
  //   // Fetch data from your API here.
  //
  // }

  // onMounted(async () => {
  //   data.value = await getData()
  // })
</script>

<template>
  <section
    class="flex flex-col rounded-md border bg-card text-card-foreground mt-5"
  >
    <div class="flex items-center justify-between px-6 py-4 border-b">
      <div class="flex items-center gap-3">
        <h2 class="text-lg font-semibold tracking-tight">Command Registry</h2>
        <Badge variant="secondary">8 active</Badge>
      </div>
    </div>
    <div class="relative w-full overflow-auto">
      <DataTable :data="data" :columns="columns" />
    </div>
  </section>
</template>

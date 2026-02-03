<script setup lang="ts" generic="TData, TValue">
import type {ColumnDef} from '@tanstack/vue-table'
import {FlexRender, getCoreRowModel, useVueTable,} from '@tanstack/vue-table'

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table/index.ts'

const props = defineProps<{
  columns: ColumnDef<TData, TValue>[]
  data: TData[]
  emptyMessage?: string
}>()

const table = useVueTable({
  get data() {
    return props.data
  },
  get columns() {
    return props.columns
  },
  getCoreRowModel: getCoreRowModel(),
})
</script>

<template>
  <div class="rounded-lg">
    <Table class="min-w-100">
      <TableHeader>
        <TableRow
            v-for="headerGroup in table.getHeaderGroups()"
            :key="headerGroup.id"
            class="bg-muted/25 hover:bg-muted/25"
        >
          <TableHead
              v-for="header in headerGroup.headers"
              :key="header.id"
              class="table-header"
          >
            <FlexRender
                v-if="!header.isPlaceholder"
                :render="header.column.columnDef.header"
                :props="header.getContext()"
            />
          </TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        <template v-if="table.getRowModel().rows?.length">
          <TableRow
              v-for="row in table.getRowModel().rows" :key="row.id"
              :data-state="row.getIsSelected() ? 'selected' : undefined"
          >
            <TableCell
                v-for="cell in row.getVisibleCells()"
                :key="cell.id"
                class="px-4 py-3"
            >
              <FlexRender :render="cell.column.columnDef.cell" :props="cell.getContext()"/>
            </TableCell>
          </TableRow>
        </template>
        <template v-else>
          <TableRow>
            <TableCell :colspan="columns.length" class="h-24 text-center">
              {{ emptyMessage ?? "No results." }}
            </TableCell>
          </TableRow>
        </template>
        
      </TableBody>
    </Table>
  </div>
</template>

<style scoped>
.table-header {
  text-align: left;
  padding: 12px 20px;
  font-size: 11px;
  text-transform: uppercase;
  color: #666;
  font-weight: 600;
  letter-spacing: 0.5px;
}
</style>

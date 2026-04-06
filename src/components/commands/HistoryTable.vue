<script setup lang="ts">
  import { useTimeAgo } from "@vueuse/core";
  import { computed, ref, watch } from "vue";
  import { Badge } from "@/components/ui/badge";
  import {
    Pagination,
    PaginationContent,
    PaginationEllipsis,
    PaginationItem,
    PaginationNext,
    PaginationPrevious,
  } from "@/components/ui/pagination";
  import { Skeleton } from "@/components/ui/skeleton";
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from "@/components/ui/table";
  import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
  } from "@/components/ui/tooltip";
  import { IExecutionHistory, TTriggeredBy } from "@/lib/api/api.types.ts";
  import { useGetCommandExecutionHistory } from "@/lib/api/composables/history.ts";
  import { formatDuration, useFormatDateTime } from "@/lib/utils.ts";
  import StatusBadge from "@/pages/components/browse/StatusBadge.vue";

  const props = withDefaults(
    defineProps<{
      commandId: number;
      showPagination?: boolean;
      pageSize?: number;
    }>(),
    { showPagination: true, pageSize: 20 }
  );

  const currentPage = ref(1);

  const { data: history, isLoading } = useGetCommandExecutionHistory(
    props.commandId
  );

  const totalPages = computed(() =>
    Math.max(1, Math.ceil((history.value?.length ?? 0) / props.pageSize))
  );

  const pagedRows = computed(() => {
    const start = (currentPage.value - 1) * props.pageSize;
    return history.value?.slice(start, start + props.pageSize);
  });

  watch(
    () => history.value?.length,
    () => {
      currentPage.value = 1;
    }
  );

  const expandedRowId = ref<number | null>(null);

  function toggleExpand(row: IExecutionHistory) {
    expandedRowId.value = expandedRowId.value === row.id ? null : row.id;
  }

  function triggeredByClass(t: TTriggeredBy): string {
    switch (t) {
      case "manual":
        return "bg-blue-500/10 text-blue-700 dark:text-blue-400 border-blue-500/20";
      case "workflow":
        return "bg-violet-500/10 text-violet-700 dark:text-violet-400 border-violet-500/20";
      case "schedule":
        return "bg-orange-500/10 text-orange-700 dark:text-orange-400 border-orange-500/20";
      default:
        return "";
    }
  }

  function exitCodeClass(code: number): string {
    return code === 0
      ? "bg-green-500/10 text-green-700 dark:text-green-400 border-green-500/20"
      : "bg-red-500/10 text-red-700 dark:text-red-400 border-red-500/20";
  }
</script>

<template>
  <div class="flex flex-col min-h-0 h-full">
    <div class="flex-1 overflow-auto min-h-0">
      <Table>
        <TableHeader>
          <TableRow class="hover:bg-transparent border-b">
            <TableHead>Triggered By</TableHead>
            <TableHead>Status</TableHead>
            <TableHead>Started</TableHead>
            <TableHead>Duration</TableHead>
            <TableHead>Exit</TableHead>
          </TableRow>
        </TableHeader>

        <TableBody>
          <template v-if="isLoading">
            <TableRow v-for="i in 8" :key="i" class="h-9">
              <TableCell :colspan="9" class="p-0">
                <Skeleton class="h-9 w-full rounded-none" />
              </TableCell>
            </TableRow>
          </template>

          <TableRow
            v-else-if=" pagedRows && !pagedRows.length"
            class="hover:bg-transparent"
          >
            <TableCell
              :colspan="9"
              class="py-10 text-center text-xs text-muted-foreground"
            >
              No executions found
            </TableCell>
          </TableRow>

          <template v-else>
            <template v-for="row in pagedRows" :key="row.id">
              <TableRow
                class="hover:bg-muted/30 border-b border-border/50 h-9"
                @click="toggleExpand(row)"
              >
                <TableCell class="py-1.5">
                  <Badge
                    variant="outline"
                    :class="['font-normal', triggeredByClass(row.triggeredBy)]"
                  >
                    {{ row.triggeredBy }}
                  </Badge>
                </TableCell>

                <TableCell class="py-1.5">
                  <StatusBadge :status="row.status" />
                </TableCell>

                <TableCell class="py-1.5">
                  <Tooltip>
                    <TooltipTrigger as-child>
                      <span
                        class="text-xs text-muted-foreground cursor-default"
                      >
                        {{ useTimeAgo(row.startedAt??"") }}
                      </span>
                    </TooltipTrigger>
                    <TooltipContent>
                      {{ useFormatDateTime(row.startedAt) }}
                    </TooltipContent>
                  </Tooltip>
                </TableCell>

                <TableCell class="py-1.5">
                  <span
                    class="font-mono text-xs text-muted-foreground tabular-nums"
                  >
                    {{ formatDuration(new Date(row.completedAt ?? "").getTime()- new Date(row.startedAt??"").getTime()) }}
                  </span>
                </TableCell>

                <TableCell class="py-1.5">
                  <Badge
                    v-if="row.exitCode !== undefined && row.exitCode !== null"
                    variant="outline"
                    :class="['font-mono text-[10px] px-1.5 py-0 h-4', exitCodeClass(row.exitCode)]"
                  >
                    {{ row.exitCode }}
                  </Badge>
                  <span v-else class="text-xs text-muted-foreground">—</span>
                </TableCell>
              </TableRow>
            </template>
          </template>
        </TableBody>
      </Table>
    </div>

    <div
      v-if="showPagination && totalPages > 1"
      class="flex items-center justify-between  py-2 border-t border-border shrink-0"
    >
      <span class="w-full text-xs text-muted-foreground">
        {{ history?.length.toLocaleString() }} total · page {{ currentPage }} of
        {{ totalPages }}
      </span>

      <Pagination
        class="w-min"
        v-model:page="currentPage"
        :items-per-page="pageSize"
        :total="history?.length ?? 0"
        :sibling-count="1"
      >
        <PaginationContent v-slot="{ items }">
          <PaginationPrevious />

          <template v-for="(item, index) in items" :key="index">
            <PaginationItem
              v-if="item.type === 'page'"
              :value="item.value"
              :is-active="item.value === currentPage"
            >
              {{ item.value }}
            </PaginationItem>
            <PaginationEllipsis
              v-else-if="item.type === 'ellipsis'"
              :index="index"
            />
          </template>

          <PaginationNext />
        </PaginationContent>
      </Pagination>
    </div>
  </div>
</template>

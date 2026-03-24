<script setup lang="ts">
  import { computed } from "vue";
  import { TExecutionStatus } from "@/lib/api/api.types.ts";

  const props = defineProps<{
    status?: TExecutionStatus;
    pid?: number;
  }>();

  const colorValueMap: Record<string, string> = {
    running: "#22c55e", // green-500
    stopping: "#eab308", // yellow-500
    failed: "#ef4444", // red-500
    success: "#22c55e",
    cancelled: "#6b7280", // gray-500
    skipped: "#6b7280",
  };

  const statusColor = computed(() => colorValueMap[props.status] ?? "#9ca3af");
</script>

<template>
  <span
    v-if="status"
    class="inline-flex items-center gap-1.5 cursor-default select-none rounded-2xl px-2 py-0.5"
    :title="status"
    :style="{ backgroundColor: `${statusColor}1a` }"
  >
    <span
      class="inline-block w-1.75 h-1.75 rounded-full shrink-0"
      :class="[{ 'status-dot--pulse': status === 'running' }]"
      :style="{ backgroundColor: `${statusColor}` }"
    />
    <span
      v-if="status !== 'running'"
      class="text-[11px] font-medium tracking-wide text-gray-500 dark:text-gray-400"
    >
      {{ status }}
    </span>

    <span
      v-else-if="pid && status === 'running'"
      class="text-[11px] font-medium tracking-tight font-mono"
    >
      {{ `PID ${pid}` }}
    </span>
  </span>
</template>

<style scoped>
  /* Pulse animation for running - kept as is since it's complex */
  .status-dot--pulse {
    box-shadow: 0 0 0 0 rgba(34, 197, 94, 0.6);
    animation: pulse-ring 1.8s ease-out infinite;
  }

  @keyframes pulse-ring {
    0% {
      box-shadow: 0 0 0 0 rgba(34, 197, 94, 0.5);
      scale: 0.8;
    }
    70% {
      box-shadow: 0 0 0 6px rgba(34, 197, 94, 0);
      scale: 0.9;
    }
    100% {
      box-shadow: 0 0 0 0 rgba(34, 197, 94, 0);
      scale: 1;
    }
  }
</style>

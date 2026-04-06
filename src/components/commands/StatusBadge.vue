<script setup lang="ts">
  import { computed } from "vue";
  import { TExecutionStatus } from "@/lib/api/api.types.ts";

  const props = withDefaults(
    defineProps<{
      status: TExecutionStatus;
      variant?: "default" | "dot-only";
    }>(),
    { variant: "default" }
  );

  interface StatusStyle {
    dot: string;
    badge: string;
    label: string;
  }

  const STYLES: Record<string, StatusStyle> = {
    running: {
      dot: "bg-amber-500",
      badge:
        "bg-amber-500/15 text-amber-600 dark:text-amber-400 border-amber-500/20",
      label: "Running",
    },
    success: {
      dot: "bg-emerald-500",
      badge:
        "bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 border-emerald-500/20",
      label: "Success",
    },
    failed: {
      dot: "bg-red-500",
      badge: "bg-red-500/15 text-red-600 dark:text-red-400 border-red-500/20",
      label: "Failed",
    },
    timeout: {
      dot: "bg-orange-500",
      badge:
        "bg-orange-500/15 text-orange-600 dark:text-orange-400 border-orange-500/20",
      label: "Timed Out",
    },
    cancelled: {
      dot: "bg-yellow-500",
      badge:
        "bg-yellow-500/15 text-yellow-600 dark:text-yellow-400 border-yellow-500/20",
      label: "Cancelled",
    },
    skipped: {
      dot: "bg-slate-400",
      badge:
        "bg-slate-500/15 text-slate-600 dark:text-slate-400 border-slate-500/20",
      label: "Skipped",
    },
    paused: {
      dot: "bg-violet-500",
      badge:
        "bg-violet-500/15 text-violet-600 dark:text-violet-400 border-violet-500/20",
      label: "Paused",
    },
    default: {
      dot: "bg-slate-400",
      badge:
        "bg-slate-500/15 text-slate-600 dark:text-slate-400 border-slate-500/20",
      label: "Completed",
    },
  };

  const style = computed(() => STYLES[props.status] ?? STYLES.default);
</script>

<template>
  <span
    v-if="variant === 'dot-only'"
    class="status-dot"
    :class="style.dot"
    :title="style.label"
  />

  <span v-else class="status-badge" :class="style.badge">
    <span class="status-dot-inner" :class="style.dot" />
    {{ style.label }}
  </span>
</template>

<style scoped>
  .status-dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .status-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 2px 8px;
    border-radius: 9999px;
    border-width: 1px;
    font-size: 0.625rem;
    font-weight: 500;
  }

  .status-dot-inner {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }
</style>

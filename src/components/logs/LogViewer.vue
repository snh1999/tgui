<script setup lang="ts">
  import { computed, nextTick, ref, watch } from "vue";
  import LogLine from "@/components/logs/LogLine.vue";
  import type { ILogLine } from "@/lib/api/api.types.ts";
  import { useExecutionStore } from "@/stores/execution.store.ts";

  const props = defineProps<{
    executionId: number | null;
    logs: ILogLine[];
  }>();

  const scrollRef = ref<HTMLElement | null>(null);

  const isRunning = computed(() => {
    if (!props.executionId) {
      return false;
    }
    const store = useExecutionStore();
    return store.getExecution(props.executionId)?.status === "running";
  });

  watch(
    () => props.logs.length,
    async () => {
      await nextTick();
      if (scrollRef.value) {
        // scrollTop assignment is synchronous and doesn't trigger reflow loops
        scrollRef.value.scrollTop = scrollRef.value.scrollHeight;
      }
    }
  );
</script>

<template>
  <section class="log-stream">
    <div class="log-header">
      <div class="log-title">
        LIVE LOG STREAM
        <span v-if="isRunning" class="live-indicator" />
        <span
          v-else-if="executionId && logs.length"
          class="text-muted text-xs font-normal normal-case"
        >
          (last run)
        </span>
      </div>
      <a href="#" class="log-link">View Full Logs</a>
    </div>

    <div class="log-container bg-muted/80 text-foreground" ref="scrollRef">
      <template v-if="logs.length">
        <LogLine
          v-for="(line, i) in logs"
          :key="line.timestamp + '-' + i"
          :timestamp="line.timestamp"
          :type="line.isStderr? 'error': 'log'"
          :message="line.content"
        />
      </template>
      <p v-else class="log-tail">No output yet…</p>
    </div>
  </section>
</template>

<style scoped>
  .log-stream {
    padding: var(--space-lg, 30px);
  }

  .log-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
  }

  .log-title {
    font-size: 14px;
    text-transform: uppercase;
    color: #666;
    font-weight: 600;
    letter-spacing: 0.5px;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .live-indicator {
    width: 8px;
    height: 8px;
    background: #10b981;
    border-radius: 50%;
    animation: pulse 2s infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }

  .log-link {
    color: #10b981;
    text-decoration: none;
    font-size: 13px;
    font-weight: 500;
  }

  .log-tail {
    color: #6b7280;
    font-style: italic;
    margin-top: 8px;
  }

  .log-container {
    border-radius: 12px;
    padding: 20px;
    font-family: "Courier New", monospace;
    font-size: 13px;
    overflow-x: auto;
    /* Give the container a capped height so it actually scrolls vertically */
    max-height: 480px;
    overflow-y: auto;
  }
</style>

import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type {
  IExecutionHistory,
  ILogLine,
  TExecutionStatus,
} from "@/lib/api/api.types.ts";

export interface IExecution extends IExecutionHistory {
  commandName: string;
  logs: ILogLine[];
  lastAccessedAt: string;
}

export const useExecutionStore = defineStore("execution", () => {
  const executions = ref<Map<number, IExecution>>(new Map());

  const allExecutions = computed(() =>
    Array.from(executions.value.values()).sort(
      (a, b) =>
        new Date(b.startedAt ?? 0).getTime() -
        new Date(a.startedAt ?? 0).getTime()
    )
  );

  const runningExecutions = computed(() =>
    allExecutions.value.filter(
      (e) => e.status === "running" || e.status === "stopping"
    )
  );

  const pendingStops = ref<
    Map<
      number,
      {
        exitCode?: number;
        timestamp: string;
        status?: TExecutionStatus;
        bufferedAt: number;
      }
    >
  >(new Map());

  const getExecution = (id: number) => executions.value.get(id);
  const getExecutionLogs = (id: number) => executions.value.get(id)?.logs ?? [];

  const getLastExecutionForCommand = (commandId: number) =>
    allExecutions.value.find((e) => e.commandId === commandId);

  const isCommandRunning = (commandId: number) =>
    runningExecutions.value.some((e) => e.commandId === commandId);

  function addExecution(execution: IExecution) {
    executions.value.set(execution.id, execution);
    // A stop event that arrived before this execution was registered
    const pending = pendingStops.value.get(execution.id);
    if (pending) {
      markProcessStopped(
        execution.id,
        pending.exitCode,
        pending.timestamp,
        pending.status
      );
      pendingStops.value.delete(execution.id);
    }
  }

  function removeExecution(id: number) {
    executions.value.delete(id);
  }

  function appendLog(
    executionId: number,
    timestamp: string,
    content: string,
    isStderr: boolean
  ) {
    const execution = executions.value.get(executionId);
    if (!execution) {
      return;
    }

    execution.logs.push({
      timestamp,
      isStderr,
      content,
      executionId,
    });

    if (execution.logs.length > 5000) {
      execution.logs.splice(0, execution.logs.length - 5000);
    }

    execution.lastAccessedAt = new Date().toISOString();
  }

  function appendLogBatch(logs: ILogLine[]) {
    if (!logs.length) {
      return;
    }

    // Group by executionId in case a batch ever spans multiple executions
    const byExecution = new Map<number, ILogLine[]>();
    for (const log of logs) {
      const group = byExecution.get(log.executionId) ?? [];
      group.push(log);
      byExecution.set(log.executionId, group);
    }

    for (const [executionId, group] of byExecution) {
      const execution = executions.value.get(executionId);
      if (!execution) {
        continue;
      }

      execution.logs.push(...group);

      if (execution.logs.length > 5000) {
        execution.logs.splice(0, execution.logs.length - 5000);
      }

      execution.lastAccessedAt = new Date().toISOString();
    }
  }

  function markProcessStarted(executionId: number, pid: number) {
    const execution = executions.value.get(executionId);
    if (!execution) {
      return;
    }

    execution.pid = pid;
    execution.status = "running";
  }

  function markProcessStopped(
    executionId: number,
    exitCode: number | undefined,
    timestamp: string,
    status?: TExecutionStatus
  ) {
    const execution = executions.value.get(executionId);
    if (!execution) {
      // Race condition: stopped event arrived before addExecution was called.
      // Buffering it, addExecution will apply it when the execution is registered.
      pendingStops.value.set(executionId, {
        exitCode,
        timestamp,
        status,
        bufferedAt: Date.now(),
      });
      return;
    }

    execution.status = status ?? (exitCode === 0 ? "success" : "failed");
    execution.exitCode = exitCode;
    execution.completedAt = timestamp;
  }

  function updateProcessStatus(
    executionId: number,
    newStatus: TExecutionStatus
  ) {
    const execution = executions.value.get(executionId);
    if (!execution) {
      return;
    }

    execution.status = newStatus;
  }

  function setProcessStopping(
    executionId: number,
    force: boolean,
    timestamp: string
  ) {
    const execution = executions.value.get(executionId);
    if (!execution) {
      return;
    }

    execution.status = "stopping";
    execution.logs.push({
      timestamp,
      isStderr: false,
      content: force
        ? "Force killing process..."
        : "Gracefully stopping process...",
      executionId,
    });
  }

  function clearOldExecutions(maxAgeHours = 24) {
    const cutoff = Date.now() - maxAgeHours * 60 * 60 * 1000;
    const terminal = new Set<TExecutionStatus>([
      "success",
      "failed",
      "cancelled",
      "timeout",
      "skipped",
      "completed",
    ]);

    for (const [id, exec] of executions.value) {
      if (
        terminal.has(exec.status) &&
        new Date(exec.lastAccessedAt).getTime() < cutoff
      ) {
        executions.value.delete(id);
      }
    }

    const cutoffMs = Date.now() - 60 * 1000;
    for (const [id, stop] of pendingStops.value) {
      if (stop.bufferedAt < cutoffMs) {
        pendingStops.value.delete(id);
      }
    }
  }

  function clearLogs(executionId: number) {
    const execution = executions.value.get(executionId);
    if (execution) {
      execution.logs = [];
    }
  }

  function $reset() {
    executions.value.clear();
    pendingStops.value.clear();
  }

  return {
    executions,
    allExecutions,
    runningExecutions,
    getExecution,
    getExecutionLogs,
    getLastExecutionForCommand,
    isCommandRunning,
    addExecution,
    removeExecution,
    appendLog,
    appendLogBatch,
    markProcessStarted,
    markProcessStopped,
    updateProcessStatus,
    setProcessStopping,
    clearOldExecutions,
    clearLogs,
    $reset,
  };
});

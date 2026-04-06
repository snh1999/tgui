/** biome-ignore-all lint/complexity/noForEach: <easier to type> */
import { listen } from "@tauri-apps/api/event";
import { processHandlerApi } from "@/lib/api/api.tauri.ts";
import type {
  ILogLine,
  TExecutionStatus,
  TProcessStatus,
} from "@/lib/api/api.types.ts";
import { saveExecutionLogs } from "@/lib/log-persistence.ts";
import { useExecutionStore } from "@/stores/execution.store";

let unlisteners: Array<() => void> = [];

function normalizeStatus(status: TProcessStatus): TExecutionStatus {
  if (status === "Idle") {
    return "idle";
  }

  switch (status.type) {
    case "stopping":
      return "stopping";
    case "stopped":
      return "completed";
    case "error":
      return "failed";
    case "running":
      return "running";
    default:
      return "failed";
  }
}

export async function initExecutionEvents() {
  const store = useExecutionStore();
  unlisteners.push(
    await listen<ILogLine[]>("log-batch", ({ payload }) => {
      if (!(Array.isArray(payload) && payload.length)) {
        return;
      }
      store.appendLogBatch(payload);
    })
  );

  unlisteners.push(
    await listen<ILogLine>("log-line", ({ payload }) => {
      store.appendLogBatch([payload]);
    })
  );

  unlisteners.push(
    await listen<{
      executionId: number;
      pid: number;
      commandId: number;
      commandName: string;
      timestamp: string;
    }>("process-started", ({ payload }) => {
      store.markProcessStarted(payload.executionId, payload.pid);
    })
  );

  unlisteners.push(
    await listen<{
      executionId: number;
      pid: number;
      exitCode?: number;
      status: TProcessStatus;
      timestamp: string;
    }>("process-stopped", ({ payload }) => {
      store.markProcessStopped(
        payload.executionId,
        payload.exitCode,
        payload.timestamp,
        normalizeStatus(payload.status)
      );

      const execution = store.getExecution(payload.executionId);
      if (execution?.commandId) {
        saveExecutionLogs(execution.commandId, {
          executionId: execution.id,
          commandId: execution.commandId,
          commandName: execution.commandName,
          status: execution.status,
          startedAt: execution.startedAt,
          completedAt: execution.completedAt,
          savedAt: new Date().toISOString(),
          logs: execution.logs,
        });
      }
    })
  );

  unlisteners.push(
    await listen<{
      executionId: number;
      oldStatus: string;
      newStatus: TProcessStatus;
      timestamp: string;
    }>("process-status-changed", ({ payload }) => {
      store.updateProcessStatus(
        payload.executionId,
        normalizeStatus(payload.newStatus)
      );
    })
  );

  // On app start, reconnect to any processes that were already running
  try {
    const running = await processHandlerApi.getRunningProcess();

    for (const proc of running) {
      if (store.getExecution(proc.executionId)) {
        continue;
      }

      store.addExecution({
        id: proc.executionId,
        commandId: proc.commandId,
        workflowId: undefined,
        workflowStepId: undefined,
        pid: proc.pid,
        status:
          normalizeStatus(proc.status) === "running" ? "running" : "stopping",
        exitCode: proc.exitCode ?? undefined,
        startedAt: proc.startTime,
        completedAt: undefined,
        triggeredBy: "manual",
        context: undefined,
        commandName: proc.commandName,
        logs: [
          {
            timestamp: new Date().toISOString(),
            isStderr: false,
            content: `Reconnected to running process (PID: ${proc.pid})`,
            executionId: proc.executionId,
          },
        ],
        lastAccessedAt: new Date().toISOString(),
      });

      // Backfill whatever the BE log buffer still has
      const logs = await processHandlerApi.getLogBuffer(
        proc.executionId,
        0,
        10_000
      );
      if (logs.length) {
        store.appendLogBatch(logs);
      }
    }
  } catch (error) {
    console.error(
      "[ExecutionEvents] Failed to reconnect to running processes:",
      error
    );
  }
}

export function cleanupExecutionEvents() {
  unlisteners.forEach((fn) => {
    fn();
  });
  unlisteners = [];
}

import type { TExecutionStatus } from "@/lib/api/api.types";
import type { LogEntry } from "@/stores/execution.store";

const KEY_PREFIX = "exec_logs:";
const MAX_PERSISTED_LOGS = 5000;

export interface PersistedExecution {
  executionId: number;
  commandId: number;
  commandName: string;
  status: TExecutionStatus;
  startedAt?: string;
  completedAt?: string;
  savedAt: string;
  logs: LogEntry[];
}

/**
 * Persist the latest execution for a command.
 * Overwrites any previous entry for the same commandId — we only keep one.
 */
export function saveExecutionLogs(
  commandId: number,
  data: PersistedExecution
): void {
  try {
    const payload: PersistedExecution = {
      ...data,
      logs: data.logs.slice(-MAX_PERSISTED_LOGS),
    };
    localStorage.setItem(KEY_PREFIX + commandId, JSON.stringify(payload));
  } catch (e) {
    // Storage full or unavailable — fail silently
    console.warn("[log-persistence] Failed to save execution logs:", e);
  }
}

/**
 * Load the persisted execution for a command, or null if none exists.
 */
export function loadExecutionLogs(
  commandId: number
): PersistedExecution | null {
  try {
    const raw = localStorage.getItem(KEY_PREFIX + commandId);
    return raw ? (JSON.parse(raw) as PersistedExecution) : null;
  } catch {
    return null;
  }
}

export function clearExecutionLogs(commandId: number): void {
  localStorage.removeItem(KEY_PREFIX + commandId);
}

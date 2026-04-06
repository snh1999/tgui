import {
  LOG_KEY_PREFIX,
  LOG_PERSISTENCE_VERSION,
  MAX_PERSISTED_LOGS,
} from "@/app.constants.ts";
import type { ILogLine, TExecutionStatus } from "@/lib/api/api.types";

export interface PersistedExecution {
  _v: number;
  executionId: number;
  commandId: number;
  commandName: string;
  status: TExecutionStatus;
  startedAt?: string;
  completedAt?: string;
  savedAt: string;
  logs: ILogLine[];
}

/**
 * Persist the latest execution for a command.
 * Overwrites any previous entry for the same commandId — we only keep one.
 */
export function saveExecutionLogs(
  commandId: number,
  data: Omit<PersistedExecution, "_v">
): void {
  try {
    const payload: PersistedExecution = {
      _v: LOG_PERSISTENCE_VERSION,
      ...data,
      logs: data.logs.slice(-MAX_PERSISTED_LOGS),
    };
    localStorage.setItem(LOG_KEY_PREFIX + commandId, JSON.stringify(payload));
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
    const raw = localStorage.getItem(LOG_KEY_PREFIX + commandId);
    if (!raw) {
      return null;
    }
    const parsed = JSON.parse(raw) as PersistedExecution;

    if (parsed._v !== LOG_PERSISTENCE_VERSION) {
      console.log(
        `[log-persistence] Discarding v${parsed._v} data for command ${commandId}`
      );
      localStorage.removeItem(LOG_KEY_PREFIX + commandId);
      return null;
    }

    return parsed;
  } catch {
    return null;
  }
}

export function clearExecutionLogs(commandId: number): void {
  localStorage.removeItem(LOG_KEY_PREFIX + commandId);
}

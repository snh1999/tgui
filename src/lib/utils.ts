import type { ClassValue } from "clsx";
import { clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import { toast } from "vue-sonner";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function toastError(error: string | Error) {
  toast.error(typeof error === "string" ? error : error.message);
}

/**
 *  milliseconds to a human-readable string (1m 23s, 1h 1m).
 */
export function formatDuration(ms: number): string {
  if (ms < 0) {
    return "—";
  }
  const totalSeconds = Math.floor(ms / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  if (minutes > 0) {
    return `${minutes}m ${seconds}s`;
  }
  return `${seconds}s`;
}

/**
 * Format a datetime string as a relative "X ago" label. ("3 min ago", "2h ago", "4d ago")
 */
export function formatRelativeTime(dateStr: string): string {
  const date = new Date(dateStr);
  const diff = Date.now() - date.getTime();

  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(diff / 60_000);
  const hours = Math.floor(diff / 3_600_000);
  const days = Math.floor(diff / 86_400_000);

  if (seconds < 10) {
    return "just now";
  }
  if (minutes < 1) {
    return `${seconds}s ago`;
  }
  if (minutes < 60) {
    return `${minutes}m ago`;
  }
  if (hours < 24) {
    return `${hours}h ago`;
  }
  return `${days}d ago`;
}

/**
 * Format a datetime string into ("Mar 15, 2026, 3:42 PM)"
 */
export function formatAbsoluteDateTime(dateStr?: string): string {
  if (!dateStr) {
    return "-";
  }
  return new Intl.DateTimeFormat("en-US", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(dateStr));
}

/**
 * Compute duration between two ISO datetime strings.
 * Returns '—' if completedAt is null/undefined (still running).
 */
export function computeDurationBetween(
  startedAt: string,
  completedAt?: string | null
): string {
  if (!completedAt) {
    return "—";
  }
  const ms = new Date(completedAt).getTime() - new Date(startedAt).getTime();
  return formatDuration(ms);
}

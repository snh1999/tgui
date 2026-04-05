import { useDateFormat } from "@vueuse/core";
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

export function useFormatDateTime(date?: string) {
  return useDateFormat(date, "MMM D, YYYY, h:mm A");
}

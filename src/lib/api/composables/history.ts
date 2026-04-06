import { useMutation, useQuery } from "@tanstack/vue-query";
import { type MaybeRef, unref } from "vue";
import { toast } from "vue-sonner";
import { queryKeys } from "@/lib/api/api.keys.ts";
import { executionHistoryApi } from "@/lib/api/api.tauri.ts";
import type { StatsTarget } from "@/lib/api/api.types.ts";
import { toastError } from "@/lib/utils.ts";

export function useGetCommandExecutionHistory(commandId: MaybeRef<number>) {
  return useQuery({
    queryKey: [...queryKeys.commands.detail(unref(commandId)), "history"],
    queryFn: () =>
      executionHistoryApi.getCommandExecutionHistory(unref(commandId)),
    enabled: unref(commandId) > 0,
  });
}

export function useGetRunningCommands() {
  return useQuery({
    queryKey: [...queryKeys.commands.lists(), "recent"],
    queryFn: () => executionHistoryApi.getRunningCommands(),
  });
}

export function useCleanupCommandHistory() {
  return useMutation({
    mutationFn: ({
      commandId,
      keepLast,
    }: {
      commandId: number;
      keepLast: number;
    }) => executionHistoryApi.cleanupCommandHistory(commandId, keepLast),
    onSuccess: () => toast.success("Cleared history"),
    onError: toastError,
  });
}

export function useCleanupHistoryOlderThan() {
  return useMutation({
    mutationFn: (days: number) =>
      executionHistoryApi.cleanupHistoryOlderThan(days),
    onSuccess: () => toast.success("Cleared history"),
    onError: toastError,
  });
}

export function useGetExecutionStats(
  target: StatsTarget,
  days?: MaybeRef<number>
) {
  return useQuery({
    queryKey: [...queryKeys.executionHistories.all, "stats"],
    queryFn: () => executionHistoryApi.getStats(target, unref(days)),
  });
}

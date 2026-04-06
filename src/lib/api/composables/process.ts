import { useMutation, useQuery } from "@tanstack/vue-query";
import { toast } from "vue-sonner";
import { queryKeys } from "@/lib/api/api.keys.ts";
import { processHandlerApi } from "@/lib/api/api.tauri.ts";
import { toastError } from "@/lib/utils.ts";

export function useResolveCommandContext(commandId: number) {
  return useQuery({
    queryKey: [...queryKeys.commands.detail(commandId), "context"],
    queryFn: () => {
      return processHandlerApi.resolveCommandContext(commandId);
    },
  });
}

export function useSpawnCommand() {
  return useMutation({
    mutationFn: (commandId: number) =>
      processHandlerApi.spawnCommand(commandId),
    onSuccess: () => {
      toast.success("Spawned command");
    },
    onError: toastError,
  });
}

export function useKillProcess() {
  return useMutation({
    mutationFn: (executionId: number) =>
      processHandlerApi.killProcess(executionId),
    onSuccess: () => {
      toast.success("Process terminated");
    },
    onError: toastError,
  });
}

export function useGetRunningProcesses() {
  return useQuery({
    queryKey: ["running"],
    queryFn: () => processHandlerApi.getRunningProcess(),
  });
}

export function useGetProcessStatus(executionId: number) {
  return useQuery({
    queryKey: ["process-status", executionId],
    queryFn: () => processHandlerApi.getProcessStatus(executionId),
    staleTime: 1000,
    gcTime: 3000,
  });
}

export function useGetLogBuffer(
  executionId: number,
  offset: number,
  limit: number
) {
  return useQuery({
    queryKey: ["log-buffer", executionId, offset, limit],
    queryFn: () => processHandlerApi.getLogBuffer(executionId, offset, limit),
    staleTime: 3000,
    gcTime: 5000,
  });
}

export function useClearLogBuffer() {
  return useMutation({
    mutationFn: (executionId: number) =>
      processHandlerApi.clearLogBuffer(executionId),
  });
}

export function useKillAllProcess() {
  return useMutation({
    mutationFn: (force: boolean) => processHandlerApi.stopAllProcess(force),
  });
}

export function useGetTrayStatus() {
  return useQuery({
    queryKey: ["stats"],
    queryFn: () => processHandlerApi.getTrayStatus(),
  });
}

export function getGetValidShells() {
  return useQuery({
    queryKey: ["shells"],
    queryFn: () => processHandlerApi.getValidShells(),
  });
}

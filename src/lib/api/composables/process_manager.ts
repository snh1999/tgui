import { useMutation, useQuery } from "@tanstack/vue-query";
import { processHandlerApi } from "@/lib/api/api.tauri.ts";

export function useSpawnCommand() {
  return useMutation({
    mutationFn: (_: unknown): Promise<void> => {
      return new Promise<void>((_, _r) => {
        console.log("inside fn");
      });
    },
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

import {  useQuery } from "@tanstack/vue-query";
import { processHandlerApi } from "@/lib/api/api.tauri.ts";

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

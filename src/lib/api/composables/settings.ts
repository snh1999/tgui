import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { type MaybeRef, unref } from "vue";
import { settingsApi } from "@/lib/api/api.tauri.ts";

export function useGetSetting(key: MaybeRef<string>) {
  return useQuery(() => ({
    queryKey: ["settings", unref(key)],
    queryFn: () => settingsApi.getSetting(unref(key)),
    enabled: () => unref(key).length > 0,
  }));
}

export function useGetAllSettings() {
  return useQuery({
    queryKey: ["settings"],
    queryFn: () => settingsApi.getAllSettings(),
  });
}

export function useResetSettings() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => settingsApi.resetSettings(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["settings"] });
    },
  });
}

export function useSetSetting() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ key, value }: { key: string; value: string }) =>
      settingsApi.setSetting(key, value),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["settings", variables.key] });
      queryClient.invalidateQueries({ queryKey: ["settings"] });
    },
  });
}

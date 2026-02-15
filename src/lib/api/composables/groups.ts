import {useMutation, useQuery, useQueryClient} from "@tanstack/vue-query";

import type {MaybeRef} from "vue";
import {unref} from "vue";
import {queryKeys} from "@/lib/api/api.keys.ts";
import {groupsApi} from "@/lib/api/api.tauri.ts";
import type {
    ICommandGroupFilter,
    IMovePosition,
    TUpsertGroupPayload,
} from "@/lib/api/api.types.ts";
import {useOptimisticUpdate} from "@/lib/api/composables/helpers.ts";

export function useGetGroups(filters?: MaybeRef<ICommandGroupFilter>) {
  return useQuery({
    queryKey: queryKeys.groups.filteredList(unref(filters) ?? {}),
    queryFn: () => groupsApi.getAll(unref(filters)),
  });
}

export function useGetGroup(id: MaybeRef<number>) {
  return useQuery({
    queryKey: queryKeys.groups.detail(unref(id)),
    queryFn: () => groupsApi.getById(unref(id)),
    enabled: () => unref(id) > 0,
  });
}

export function useGetGroupTree(id: MaybeRef<number>) {
  return useQuery({
    queryKey: [queryKeys.groups.detail(unref(id)), "tree"],
    queryFn: () => groupsApi.getGroupTree(unref(id)),
    enabled: () => unref(id) > 0,
  });
}

export function useGetGroupPath(id: MaybeRef<number>) {
  return useQuery({
    queryKey: [queryKeys.groups.detail(unref(id)), "path"],
    queryFn: () => groupsApi.getGroupPath(unref(id)),
    enabled: () => unref(id) > 0,
  });
}

export function useCreateGroup() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (payload: TUpsertGroupPayload) => groupsApi.create(payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.groups.lists() });
    },
  });
}

export function useUpdateGroup() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      id,
      payload,
    }: {
      id: number;
      payload: TUpsertGroupPayload;
    }) => groupsApi.update(id, payload),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.groups.detail(variables.id),
      });
      queryClient.invalidateQueries({ queryKey: queryKeys.groups.lists() });
    },
  });
}

export function useDeleteGroup() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: number) => groupsApi.delete(id),
    onSuccess: (_, id) => {
      queryClient.removeQueries({ queryKey: queryKeys.groups.detail(id) });
      queryClient.invalidateQueries({ queryKey: queryKeys.groups.lists() });
      queryClient.invalidateQueries({ queryKey: queryKeys.commands.lists() });
    },
  });
}

export function useMoveGroup() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (options: IMovePosition) => groupsApi.move(options),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.groups.lists() });
    },
  });
}

export function useToggleFavoriteGroup() {
  return useOptimisticUpdate(
    ({ id }: { id: number }) => groupsApi.toggleFavorite(id),
    queryKeys.groups
  );
}

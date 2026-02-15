import {useMutation, useQuery, useQueryClient} from "@tanstack/vue-query";
import type {MaybeRef} from "vue";
import {unref} from "vue";
import {queryKeys} from "@/lib/api/api.keys.ts";
import {categoriesApi} from "@/lib/api/api.tauri.ts";
import type {TUpsertCategoryPayload} from "@/lib/api/api.types.ts";

export function useGetCategories() {
  return useQuery({
    queryKey: queryKeys.categories.lists(),
    queryFn: () => categoriesApi.getAll(),
  });
}

export function useGetCategory(id: MaybeRef<number>) {
  return useQuery({
    queryKey: queryKeys.categories.detail(unref(id)),
    queryFn: () => categoriesApi.getById(unref(id)),
    enabled: () => unref(id) > 0,
  });
}

export function useCategoryCommandCount(id: MaybeRef<number>) {
  return useQuery({
    queryKey: [
      ...queryKeys.categories.detail(unref(id)),
      "command-count",
    ] as const,
    queryFn: () => categoriesApi.getCommandCount(unref(id)),
    enabled: () => unref(id) > 0,
  });
}

export function useCreateCategory() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (payload: TUpsertCategoryPayload) =>
      categoriesApi.create(payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.categories.lists() });
    },
  });
}

export function useUpdateCategory() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      id,
      payload,
    }: {
      id: number;
      payload: TUpsertCategoryPayload;
    }) => categoriesApi.update(id, payload),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.categories.detail(variables.id),
      });
      queryClient.invalidateQueries({ queryKey: queryKeys.categories.lists() });
    },
  });
}

export function useDeleteCategory() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => categoriesApi.delete(id),
    onSuccess: (_, id) => {
      queryClient.removeQueries({ queryKey: queryKeys.categories.detail(id) });
      queryClient.invalidateQueries({ queryKey: queryKeys.categories.lists() });
      //   // Invalidate commands/groups that might reference this category
      // queryClient.invalidateQueries({ queryKey: queryKeys.commands.lists() });
      // queryClient.invalidateQueries({ queryKey: queryKeys.groups.lists() });
    },
  });
}

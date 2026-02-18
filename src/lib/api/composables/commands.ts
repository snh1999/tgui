import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import type { MaybeRef } from "vue";
import { unref } from "vue";
import { toast } from "vue-sonner";
import { queryKeys } from "@/lib/api/api.keys.ts";
import { commandsApi } from "@/lib/api/api.tauri.ts";
import { useOptimisticUpdate } from "@/lib/api/composables/helpers.ts";
import type {
  ICommandGroupFilter,
  IMovePosition,
  TUpsertCommandPayload,
} from "../api.types";

export function useGetCommands(filters?: MaybeRef<ICommandGroupFilter>) {
  return useQuery({
    queryKey: queryKeys.commands.filteredList(unref(filters)),
    queryFn: () => commandsApi.getAll(unref(filters)),
  });
}

export function useGetCommand(id: MaybeRef<number>) {
  return useQuery({
    queryKey: queryKeys.commands.detail(unref(id)),
    queryFn: () => commandsApi.getById(unref(id)),
    enabled: () => unref(id) > 0,
  });
}

export function useCommandSearch(searchTerm: MaybeRef<string>) {
  return useQuery({
    queryKey: [
      ...queryKeys.commands.lists(),
      "search",
      unref(searchTerm),
    ] as const,
    queryFn: () => commandsApi.search(unref(searchTerm)),
    enabled: () => unref(searchTerm).length > 0,
  });
}

export function useCreateCommand() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (payload: TUpsertCommandPayload) => commandsApi.create(payload),
    onSuccess: () => {
      toast.success("Command created!");
      queryClient.invalidateQueries({ queryKey: queryKeys.commands.lists() });
    },
    onError: (error) => {
      toast.error(typeof error === "string" ? error : error.message);
    },
  });
}

export function useUpdateCommand() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      id,
      payload,
    }: {
      id: number;
      payload: TUpsertCommandPayload;
    }) => commandsApi.update(id, payload),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.commands.detail(variables.id),
      });
      queryClient.invalidateQueries({ queryKey: queryKeys.commands.lists() });
      toast.success("Command updated!");
    },
    onError: (error) => {
      toast.error(typeof error === "string" ? error : error.message);
    },
  });
}

export function useDeleteCommand() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: number) => commandsApi.delete(id),
    onSuccess: (_, id) => {
      queryClient.removeQueries({ queryKey: queryKeys.commands.detail(id) });
      queryClient.invalidateQueries({ queryKey: queryKeys.commands.lists() });
      toast.info("Command deleted!");
    },
    onError: (error) => {
      toast.error(typeof error === "string" ? error : error.message);
    },
  });
}

export function useToggleFavoriteCommand() {
  return useOptimisticUpdate(
    ({ id }: { id: number }) => commandsApi.toggleFavorite(id),
    queryKeys.commands
  );
}

export function useMoveCommand() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (options: IMovePosition) => commandsApi.move(options),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.commands.lists() });
    },
  });
}

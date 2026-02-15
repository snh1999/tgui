/** biome-ignore-all lint/suspicious/noExplicitAny: <type can not be specified> */
import {useMutation, useQueryClient} from "@tanstack/vue-query";
import type {IQueryKeyTypes} from "@/lib/api/api.types.ts";

export function useOptimisticUpdate<T extends { id: number }>(
  mutation: (data: T) => Promise<unknown>,
  queryKeys: IQueryKeyTypes
) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: T) => mutation(data),
    onMutate: async ({ id, ...rest }) => {
      await queryClient.cancelQueries({
        queryKey: queryKeys.detail(id),
      });

      const previous = queryClient.getQueryData(queryKeys.detail(id));

      queryClient.setQueryData(queryKeys.detail(id), (old: any) => {
        if (!old) {
          return old;
        }
        return { ...old, ...rest };
      });

      return { previous, id };
    },
    onError: (_, { id }, context) => {
      // Rollback
      if (context?.previous) {
        queryClient.setQueryData(queryKeys.detail(id), context.previous);
      }
    },
    onSettled: (_, __, { id }) => {
      // Refetch to ensure sync
      queryClient.invalidateQueries({
        queryKey: queryKeys.detail(id),
      });
      queryClient.invalidateQueries({ queryKey: queryKeys.lists() });
    },
  });
}

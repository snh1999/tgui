/** biome-ignore-all lint/suspicious/noExplicitAny: <type can not be specified> */
import { useMutation, useQueryClient } from "@tanstack/vue-query";
import { toast } from "vue-sonner";
import type { IQueryKeyTypes } from "@/lib/api/api.types.ts";

export function useOptimisticUpdate<T extends { id: number }>(
  mutation: (data: T) => Promise<unknown>,
  queryKeys: IQueryKeyTypes
) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: T) => mutation(data),
    onMutate: async (data) => {
      await queryClient.cancelQueries({ queryKey: queryKeys.lists() });
      await queryClient.cancelQueries({ queryKey: queryKeys.detail(data.id) });

      const previousLists = queryClient.getQueryData(queryKeys.lists());
      const previousDetail = queryClient.getQueryData(
        queryKeys.detail(data.id)
      );

      queryClient.setQueryData(queryKeys.detail(data.id), (old: any) => {
        if (!old) {
          return old;
        }
        return { ...old, ...data };
      });

      queryClient.setQueriesData(
        { queryKey: queryKeys.lists() },
        (old: any) => {
          if (!old) {
            return old;
          }

          if (Array.isArray(old)) {
            return old.map((item: any) =>
              item.id === data.id ? { ...item, ...data } : item
            );
          }
          return old;
        }
      );
      return { previousLists, previousDetail, id: data.id };
    },
    onSuccess: (response, { id }) => {
      queryClient.setQueryData(queryKeys.detail(id), response);
    },
    onError: (e, { id }, context) => {
      toast.error(e);
      if (context?.previousLists) {
        queryClient.setQueryData(queryKeys.lists(), context.previousLists);
      }
      if (context?.previousDetail) {
        queryClient.setQueryData(queryKeys.detail(id), context.previousDetail);
      }
    },
    onSettled: (_, __, { id }) => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.detail(id),
      });
      queryClient.invalidateQueries({ queryKey: queryKeys.lists() });
    },
  });
}

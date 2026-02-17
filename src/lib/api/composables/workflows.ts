import {
  QueryClient,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/vue-query";
import type { MaybeRef } from "vue";
import { unref } from "vue";
import {
  CATEGORY_WORKFLOW_COUNT,
  COMMAND_POPULATED_STEP,
  queryKeys,
  WORKFLOW_STEP_COUNT,
} from "@/lib/api/api.keys.ts";
import { workflowsApi, workflowStepApi } from "@/lib/api/api.tauri.ts";
import type {
  ICommandGroupFilter,
  IMovePosition,
  IWorkflowStepFilter,
  TUpsertWorkflowPayload,
  TUpsertWorkflowStepsPayload,
} from "@/lib/api/api.types.ts";
import { useOptimisticUpdate } from "@/lib/api/composables/helpers.ts";

export function useGetWorkflows(filter: MaybeRef<ICommandGroupFilter>) {
  return useQuery({
    queryKey: queryKeys.workflows.filteredList(unref(filter)),
    queryFn: () => workflowsApi.getAll(unref(filter)),
  });
}

export function useGetWorkflow(id: MaybeRef<number>) {
  return useQuery({
    queryKey: queryKeys.workflows.detail(unref(id)),
    queryFn: () => workflowsApi.getById(unref(id)),
    enabled: () => unref(id) > 0,
  });
}

export function useCreateWorkflow() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (payload: TUpsertWorkflowPayload) =>
      workflowsApi.create(payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.workflows.lists() });
    },
  });
}

export function useUpdateWorkflow() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      id,
      payload,
    }: {
      id: number;
      payload: TUpsertWorkflowPayload;
    }) => workflowsApi.update(id, payload),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.workflows.detail(id),
      });
      queryClient.invalidateQueries({ queryKey: queryKeys.workflows.lists() });
    },
  });
}

export function useDeleteWorkflow() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: number) => workflowsApi.delete(id),
    onSuccess: (_, id) => {
      queryClient.removeQueries({ queryKey: queryKeys.workflows.detail(id) });
      queryClient.invalidateQueries({ queryKey: queryKeys.workflows.lists() });
    },
  });
}

export function useToggleFavoriteWorkflow() {
  return useOptimisticUpdate(
    ({ id }: { id: number }) => workflowsApi.toggleFavorite(id),
    queryKeys.workflows
  );
}

export function useMoveWorkflow() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (options: IMovePosition) => workflowsApi.move(options),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.workflows.lists() });
    },
  });
}

export function useCategoryWorkflowCount(id: MaybeRef<number>) {
  return useQuery({
    queryKey: [
      ...queryKeys.categories.detail(unref(id)),
      CATEGORY_WORKFLOW_COUNT,
    ] as const,
    queryFn: () => workflowsApi.getCountByCategory(unref(id)),
    enabled: () => unref(id) > 0,
  });
}

export function useGetWorkflowSteps(filter: MaybeRef<IWorkflowStepFilter>) {
  return useQuery({
    queryKey: queryKeys.workflowSteps.filteredList(filter),
    queryFn: () => workflowStepApi.getAll(unref(filter)),
  });
}

export function useGetWorkflowStepsWithCommand(
  filter: MaybeRef<{
    workflowId: number;
    enabledOnly?: boolean;
  }>
) {
  const { workflowId, enabledOnly } = unref(filter);
  return useQuery({
    queryKey: [
      ...queryKeys.workflowSteps.filteredList(unref(filter)),
      COMMAND_POPULATED_STEP,
    ],
    queryFn: () => workflowStepApi.getAllWithCommands(workflowId, enabledOnly),
  });
}

export function useGetWorkflowStep(id: MaybeRef<number>) {
  return useQuery({
    queryKey: queryKeys.workflowSteps.detail(unref(id)),
    queryFn: () => workflowStepApi.getById(unref(id)),
    enabled: () => unref(id) > 0,
  });
}

function workflowStepMutationOnSuccess(
  queryClient: QueryClient,
  commandId: number,
  workflowId: number
) {
  queryClient.invalidateQueries({
    queryKey: queryKeys.workflowSteps.lists(),
  });
  queryClient.invalidateQueries({
    queryKey: queryKeys.workflowSteps.filteredList({ workflowId }),
  });
  queryClient.invalidateQueries({
    queryKey: queryKeys.workflowSteps.filteredList({ commandId }),
  });
}

export function useCreateWorkflowStep() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (payload: TUpsertWorkflowStepsPayload) =>
      workflowStepApi.create(payload),
    onSuccess: (_, { commandId, workflowId }) => {
      workflowStepMutationOnSuccess(queryClient, commandId, workflowId);
    },
  });
}

export function useUpdateWorkflowStep() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      id,
      payload,
    }: {
      id: number;
      payload: TUpsertWorkflowStepsPayload;
    }) => workflowStepApi.update(id, payload),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.workflowSteps.detail(id),
      });
      queryClient.invalidateQueries({
        queryKey: queryKeys.workflowSteps.lists(),
      });
    },
  });
}

export function useDeleteWorkflowStep() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      id,
    }: {
      id: number;
      workflowId: number;
      commandId: number;
    }) => workflowStepApi.delete(id),
    onSuccess: (_, { id, workflowId, commandId }) => {
      queryClient.removeQueries({
        queryKey: queryKeys.workflowSteps.detail(id),
      });
      workflowStepMutationOnSuccess(queryClient, commandId, workflowId);
    },
  });
}

export function useMoveWorkflowStep() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (options: IMovePosition) => workflowStepApi.move(options),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: queryKeys.workflowSteps.lists(),
      });
    },
  });
}

export function useToggleEnabledWorkflowStep() {
  return useOptimisticUpdate(
    ({ id }: { id: number }) => workflowStepApi.toggleEnabled(id),
    queryKeys.workflowSteps
  );
}

export function getWorkflowStepCount(workflowId: number) {
  return useQuery({
    queryKey: [...queryKeys.workflows.detail(workflowId), WORKFLOW_STEP_COUNT],
    queryFn: () => workflowStepApi.getWorkflowStepCount(workflowId),
  });
}

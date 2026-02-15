import type {
    ICommandGroupFilter,
    IQueryKeyTypes,
    IWorkflowStepFilter,
} from "@/lib/api/api.types.ts";

type TQueryKeys =
  | "commands"
  | "groups"
  | "categories"
  | "workflows"
  | "workflowSteps";

export const queryKeys: Record<TQueryKeys, IQueryKeyTypes> = {
  commands: {
    all: ["commands"] as const,
    lists: () => [...queryKeys.commands.all, "list"] as const,
    filteredList: (filters?: ICommandGroupFilter) =>
      [...queryKeys.commands.lists(), filters] as const,
    detail: (id: number) => [...queryKeys.commands.all, "detail", id] as const,
  },

  groups: {
    all: ["groups"] as const,
    lists: () => [...queryKeys.groups.all, "list"] as const,
    filteredList: (filters: ICommandGroupFilter) =>
      [...queryKeys.groups.lists(), filters] as const,
    detail: (id: number) => [...queryKeys.groups.all, "detail", id] as const,
  },

  categories: {
    all: ["categories"] as const,
    lists: () => [...queryKeys.categories.all, "list"] as const,
    filteredList: () => [...queryKeys.categories.lists()] as const,
    detail: (id: number) =>
      [...queryKeys.categories.all, "detail", id] as const,
  },

  workflows: {
    all: ["workflows"] as const,
    lists: () => [...queryKeys.workflows.all, "list"] as const,
    filteredList: (filter: ICommandGroupFilter) =>
      [...queryKeys.workflows.lists(), filter] as const,
    detail: (id: number) => [...queryKeys.workflows.all, "detail", id] as const,
  },

  workflowSteps: {
    all: ["workflowSteps"] as const,
    lists: () => [...queryKeys.workflowSteps.all, "list"] as const,
    filteredList: (filter: IWorkflowStepFilter) => {
      const { workflowId, commandId, enabledOnly } = filter;

      let queryKey: readonly (string | number)[] = [];
      if (workflowId) {
        queryKey = queryKeys.workflows.detail(workflowId);
      } else if (commandId) {
        queryKey = queryKeys.commands.detail(commandId);
      }

      return [
        ...queryKey,
        ...queryKeys.workflowSteps.all,
        enabledOnly,
      ] as const;
    },
    // filteredList: (filter: IWorkflowStepFilter) =>
    //   [...queryKeys.workflowSteps.lists(), filter] as const,
    detail: (id: number) =>
      [...queryKeys.workflowSteps.all, "detail", id] as const,
  },
};

export const WORKFLOW_STEP_COUNT = "step-count";
export const COMMAND_POPULATED_STEP = "populated";
export const CATEGORY_WORKFLOW_COUNT = "workflow-count";

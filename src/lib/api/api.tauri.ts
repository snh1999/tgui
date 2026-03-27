import { invoke } from "@tauri-apps/api/core";
import type {
  ICategory,
  ICommand,
  ICommandGroupFilter,
  ICommandWithHistory,
  IExecutionHistory,
  IExecutionStats,
  IGroup,
  IGroupNode,
  IMovePosition,
  ITrayStatus,
  IWorkflow,
  IWorkflowFilter,
  IWorkflowStep,
  IWorkflowStepFilter,
  StatsTarget,
  TUpsertCategoryPayload,
  TUpsertCommandPayload,
  TUpsertGroupPayload,
  TUpsertWorkflowPayload,
  TUpsertWorkflowStepsPayload,
} from "@/lib/api/api.types.ts";

export const categoriesApi = {
  getAll: () => invoke<ICategory[]>("get_categories"),

  getById: (id: number) => invoke<ICategory>("get_category", { id }),

  create: (payload: TUpsertCategoryPayload) =>
    invoke<number>("create_category", { ...payload, id: 0 }),

  update: (id: number, payload: TUpsertCategoryPayload) =>
    invoke<void>("update_category", { id, ...payload }),

  delete: (id: number) => invoke<void>("delete_category", { id }),

  getCommandCount: (id: number) =>
    invoke<number>("get_category_command_count", { id }),
};

export const commandsApi = {
  getAll: (filters?: ICommandGroupFilter) =>
    invoke<ICommandWithHistory[]>("get_commands", {
      parentId: filters?.parentId ?? "All",
      categoryId: filters?.categoryId ?? "All",
      favoritesOnly: filters?.favoritesOnly ?? false,
    }),

  getRecent: (limit: number) =>
    invoke<ICommandWithHistory[]>("get_recent_commands", {
      limit,
    }),

  getById: (id: number) => invoke<ICommand>("get_command", { id }),

  create: (payload: TUpsertCommandPayload) =>
    invoke<number>("create_command", {
      cmd: {
        ...payload,
        id: 0,
      },
    }),

  update: (id: number, payload: TUpsertCommandPayload) =>
    invoke<void>("update_command", {
      cmd: { id, ...payload },
    }),

  delete: (id: number) => invoke<void>("delete_command", { id }),

  toggleFavorite: (id: number) =>
    invoke<void>("toggle_command_favorite", { id }),

  move: (movePosition: IMovePosition) =>
    invoke<void>("move_command_between", { ...movePosition }),

  search: (searchTerm: string) =>
    invoke<ICommand[]>("search_commands", { searchTerm }),
};

export const groupsApi = {
  getAll: (filters?: ICommandGroupFilter) =>
    invoke<IGroup[]>("get_groups", {
      parentId: filters?.parentId ?? "All",
      categoryId: filters?.categoryId ?? "All",
      favoritesOnly: filters?.favoritesOnly ?? false,
    }),

  getById: (id: number) => invoke<IGroup>("get_group", { id }),

  create: (payload: TUpsertGroupPayload) =>
    invoke<number>("create_group", {
      group: {
        ...payload,
        id: 0,
      },
    }),

  update: (id: number, payload: TUpsertGroupPayload) =>
    invoke<void>("update_group", { group: { id, ...payload } }),

  toggleFavorite: (id: number) => invoke<void>("toggle_group_favorite", { id }),

  delete: (id: number) => invoke<void>("delete_group", { id }),

  move: (movePosition: IMovePosition) =>
    invoke<void>("move_group_between", { ...movePosition }),

  getGroupTree: (rootId: number) =>
    invoke<IGroupNode>("get_group_tree", { rootId }),

  getGroupPath: (rootId: number) =>
    invoke<string[]>("get_group_path", { rootId }),

  getGroupCommandCount: (id: number) =>
    invoke<number>("get_groups_count", { id }),
};

export const workflowsApi = {
  getAll: (filters?: IWorkflowFilter) =>
    invoke<IWorkflow[]>("get_workflows", {
      favoritesOnly: filters?.favoritesOnly ?? false,
      categoryId: filters?.categoryId,
    }),

  getById: (id: number) => invoke<IWorkflow>("get_workflow", { id }),

  create: (payload: TUpsertWorkflowPayload) =>
    invoke<number>("create_workflow", { workflow: { ...payload, id: 0 } }),

  update: (id: number, payload: TUpsertWorkflowPayload) =>
    invoke<void>("update_workflow", { workflow: { id, ...payload } }),

  delete: (id: number) => invoke<void>("delete_workflow", { id }),

  toggleFavorite: (id: number) =>
    invoke<void>("toggle_favorite_workflow", { id }),

  move: (movePosition: IMovePosition) =>
    invoke<void>("move_workflow_between", { ...movePosition }),

  getCountByCategory: (categoryId: number) =>
    invoke<number>("get_workflow_count_for_category", { categoryId }),
};

export const workflowStepApi = {
  getAll: (filters?: IWorkflowStepFilter) =>
    invoke<IWorkflowStep[]>("get_workflow_steps", {
      workflowId: filters?.workflowId,
      commandId: filters?.commandId,
      enabledOnly: filters?.enabledOnly ?? false,
    }),

  getAllWithCommands: (workflowId: number, enabledOnly?: boolean) =>
    invoke<[IWorkflowStep, ICommand][]>(
      "get_workflow_steps_command_populated",
      {
        workflowId,
        enabledOnly,
      }
    ),

  getById: (id: number) => invoke<IWorkflowStep>("get_workflow_step", { id }),

  create: (payload: TUpsertWorkflowStepsPayload) =>
    invoke<number>("create_workflow_step", {
      flow_steps: {
        ...payload,
        id: 0,
      },
    }),

  update: (id: number, payload: TUpsertWorkflowStepsPayload) =>
    invoke<void>("update_workflow_step", { flow_steps: { id, ...payload } }),

  delete: (id: number) => invoke<void>("delete_workflow_step", { id }),

  move: (movePosition: IMovePosition) =>
    invoke<void>("move_workflow_step_between", { ...movePosition }),

  toggleEnabled: (id: number) =>
    invoke<void>("toggle_workflow_step_enabled", { id }),

  getWorkflowStepCount: (id: number) =>
    invoke<number>("get_workflow_step_count", { id }),
};

export const processHandlerApi = {
  getTrayStatus: () => invoke<ITrayStatus>("get_tray_status"),

  getValidShells: () => invoke<string[]>("get_valid_shells"),
};

export const executionHistoryApi = {
  getCommandExecutionHistory: (commandId: number) =>
    invoke<IExecutionHistory[]>("get_command_execution_history", { commandId }),

  getRunningCommands: () => invoke<IExecutionHistory[]>("get_running_commands"),

  getStats: (target: StatsTarget, days?: number) =>
    invoke<IExecutionStats>("get_execution_stats", { target, days }),
};

export const settingsApi = {
  getSetting: (key: string) => invoke<string>("get_setting", { key }),

  setSetting: (key: string, value: string) =>
    invoke<void>("set_setting", { key, value }),

  resetSettings: () => invoke<string>("reset_settings"),

  getAllSettings: () => invoke<Record<string, string>>("get_all_settings"),
};

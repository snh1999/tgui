export interface SerializedError {
  code: string;
  message: string;
}

export interface ICategory {
  id: number;
  name: string;
  icon?: string | null;
  color?: string | null;
  createdAt?: Date;
}

interface ICommonFields {
  id: number;
  createdAt?: string;
  updatedAt?: string;
  position: number;
}

interface ICommonPositionFields extends ICommonFields {
  name: string;
  description?: string;
  isFavorite?: boolean;
  position: number;
}

interface IGroupCommandCommon extends ICommonPositionFields {
  envVars?: Map<string, string>;
  categoryId?: number | null;
  workingDirectory?: string;
  shell?: string;
}

export interface ICommand extends IGroupCommandCommon {
  command: string;
  arguments?: string[];
  groupId?: number | null;
}

export interface ICommandGroupFilter {
  parentId?: number;
  categoryId?: number;
  favoritesOnly: boolean;
}

export interface IMovePosition {
  id: number;
  prevId?: number;
  nextId?: number;
}

export interface IGroup extends IGroupCommandCommon {
  parentGroupId?: number;
  icon?: string;
}

export interface IWorkflow extends ICommonPositionFields {
  categoryId?: number;
  executionMode: "sequential" | "parallel" | "conditional";
}

export interface IWorkflowStep extends ICommonPositionFields {
  workflowId: number;
  commandId: number;
  condition: "always" | "on_success" | "on_failure";
  timeoutSeconds?: number;
  autoRetryCount?: number;
  enabled: boolean;
  continueOnFailure?: boolean;
}

export interface IWorkflowStepFilter {
  workflowId?: number;
  commandId?: number;
  enabledOnly?: boolean;
}

type TUpsertPayload<T> = Omit<T, "id" | "position" | "createdAt" | "updatedAt">;

export type TUpsertCategoryPayload = TUpsertPayload<ICategory>;
export type TUpsertCommandPayload = TUpsertPayload<ICommand>;
export type TUpsertGroupPayload = TUpsertPayload<IGroup>;
export type TUpsertWorkflowPayload = TUpsertPayload<IWorkflow>;
export type TUpsertWorkflowStepsPayload = TUpsertPayload<IWorkflowStep>;

export interface IQueryKeyTypes {
  all: readonly [string];
  // biome-ignore lint/suspicious/noExplicitAny: <generic type>
  filteredList: (filter: any) => readonly unknown[];
  lists: () => readonly string[];
  detail: (id: number) => readonly (string | number)[];
}

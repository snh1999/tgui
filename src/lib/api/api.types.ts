export interface SerializedError {
  code: string;
  message: string;
}

export interface ICategory {
  id: number;
  name: string;
  icon?: string | null;
  color?: string | null;
  createdAt?: string;
}

interface ICommonFields {
  id: number;
  createdAt?: string;
  updatedAt?: string;
}

interface ICommonPositionFields extends ICommonFields {
  name: string;
  description?: string | null;
  isFavorite?: boolean;
  position: number;
}

interface IGroupCommandCommon extends ICommonPositionFields {
  envVars?: Map<string, string> | null;
  categoryId?: number | null;
  workingDirectory?: string | null;
  shell?: string | null;
}

export interface ICommand extends IGroupCommandCommon {
  command: string;
  arguments?: string[] | null;
  groupId?: number | null;
}

export interface ICommandWithHistory extends ICommand {
  history?: IExecutionHistory;
}

export type TGroupFilter = { Group: number } | "None" | "All";
export type TCategoryFilter = { Category: number } | "None" | "All";

export interface ICommandGroupFilter {
  parentId: TGroupFilter;
  categoryId: TCategoryFilter;
  favoritesOnly: boolean;
}

export interface IMovePosition {
  id: number;
  prevId?: number;
  nextId?: number;
}

export interface ISpawnContext {
  commandId: number;
  name: string;
  executable: string;
  arguments: string[];
  workingDirectory: string;
  envVars?: Map<string, string> | null;
  shell?: string | null;
}

export interface IProcessInfo {
  executionId: number;
  pid: number;
  commandId: number;
  commandName: string;
  command: string;
  status: TProcessStatus;
  startTime: string;
  exitCode?: number | null;
  logLineCount: number;
}

export type TProcessStatus =
  | "Idle"
  | { type: "running"; pid: number; startTime: string }
  | { type: "stopping"; since: string }
  | { type: "stopped"; exitCode: number; completedAt: string }
  | { type: "error"; exitCode?: number; message: string };

export interface ILogLine {
  executionId: number;
  timestamp: string;
  content: string;
  isStderr: boolean;
}

export interface IGroup extends IGroupCommandCommon {
  parentGroupId?: number | null;
  icon?: string | null;
}

export interface IGroupNode {
  group: IGroup;
  children: IGroupNode[];
}

export interface IWorkflowFilter {
  categoryId?: number;
  favoritesOnly: boolean;
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

export type TExecutionStatus =
  | "idle"
  | "running"
  | "success"
  | "paused"
  | "failed"
  | "timeout"
  | "cancelled"
  | "skipped"
  | "completed"
  | "stopping";

export type TTriggeredBy = "manual" | "workflow" | "schedule";

export interface IExecutionHistory {
  id: number;
  commandId?: number;
  workflowId?: number;
  workflowStepId?: number;
  pid?: number;
  status: TExecutionStatus;
  exitCode?: number;
  startedAt?: string;
  completedAt?: string;
  triggeredBy: TTriggeredBy;
  context?: string;
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

export interface ITrayStatus {
  runningCount: number;
  errorCount: number;
  totalCommands: number;
}

export type StatsTarget = { Command: number } | { Workflow: number } | "Global";

export interface IExecutionStats {
  totalCounts: number;
  successCount: number;
  failedCount: number;
  cancelledCount: number;
  timeoutCount: number;
  pausedCount: number;
  skippedCount: number;
  successRate: number;
  runningCount: number;
  averageDurationMs?: number;
  lastExecutedAt?: Date;
  firstExecutedAt?: Date;
}

export interface IExplainResult {
  summary: string;
  isPrivileged: boolean;
  isDestructive: boolean;
  segments: ISegmentResult[];
}

export interface ISegmentResult {
  raw: string;
  tldrDescription?: string;
  unknownParts?: string[];
  isPrivileged: boolean;
  isDestructive: boolean;
  connector?: string;
  hasRedirection: boolean;
  isBackground: boolean;
}

import { h } from "vue";
import type { ColumnDef } from "@tanstack/vue-table";
import { Button } from "@/components/ui/button";
import {
  DirectoryIcon,
  FilledStarIcon,
  MenuDotsIcon,
  PlayIcon,
  RestartIcon,
  StarIcon,
} from "@/assets/Icons.ts";

export interface Command {
  id: string;
  name: string;
  description?: string;
  activityTime: string;
  pid?: number;
  status: "running" | "stopped" | "error";
  working_directory: string;
  isFavorite?: boolean;
}

export const data: Command[] = [
  {
    id: "1",
    name: "Dev server",
    description: "npm run dev",
    activityTime: "2 minutes ago",
    pid: 45092,
    status: "running",
    working_directory: "~/dev",
    isFavorite: false,
  },
  {
    id: "2",
    name: "Postgresql",
    description: "docker-compose up",
    activityTime: "Yesterday, 18:45",
    status: "stopped",
    working_directory: "/os/home",
    isFavorite: true,
  },
  {
    id: "3",
    name: "Unit Tests",
    description: "npm run test:watch",
    activityTime: "5 seconds ago",
    status: "error",
    working_directory: "~/dev/documents/process",
    isFavorite: false,
  },
];

const statusConfig = {
  running: { dot: "bg-emerald-500" },
  stopped: { dot: "bg-gray-500" },
  error: { dot: "bg-red-500" },
};

export const columns: ColumnDef<Command>[] = [
  {
    accessorKey: "name",
    header: "Command Name",
    cell: ({ row }) => {
      const command = row.original;
      return h("div", { class: "flex items-center gap-3 min-w-0" }, [
        h(
          Button,
          {
            variant: "ghost",
            size: "icon",
            class: "h-8 w-8 shrink-0",
            // onClick: () => toggleFavorite(command.id)
          },
          [
            h(command.isFavorite ? FilledStarIcon : StarIcon, {
              class: command.isFavorite ? "text-yellow-500" : "text-foreground",
              style: { width: "20px", height: "20px" },
            }),
          ]
        ),
        h("div", { class: "flex flex-col min-w-0" }, [
          h(
            "span",
            { class: "font-semibold mb-1 text-base truncate" },
            command.name
          ),
          command.description
            ? h(
                "span",
                {
                  class: "text-xs tracking-wide text-muted-foreground truncate",
                },
                command.description
              )
            : null,
        ]),
      ]);
    },
  },
  {
    accessorKey: "status",
    header: "Status",
    cell: ({ row }) => {
      const status = row.original.status;
      const style = statusConfig[status] ?? { dot: "bg-gray-500" };

      return h("div", { class: "flex flex-col gap-1" }, [
        h("div", { class: "flex items-center gap-2" }, [
          h("span", { class: `h-2 w-2 rounded-full ${style.dot}` }),
          h("span", { class: "text-sm text-xs mb-1 capitalize" }, status),
        ]),
        h(
          "div",
          { class: "flex items-center gap-1.5 text-xs text-muted-foreground" },
          [
            h(DirectoryIcon, { class: "h-3.5 w-3.5 shrink-0" }),
            h("span", { class: "truncate" }, row.original.working_directory),
          ]
        ),
      ]);
    },
  },
  {
    accessorKey: "activityTime",
    header: "Last Execution",
    cell: ({ row }) => {
      const command = row.original;
      return h("div", { class: "flex flex-col gap-0.5" }, [
        h(
          "span",
          { class: "text-xs text-muted-foreground" },
          command.activityTime
        ),
        command.pid
          ? h(
              "span",
              { class: "text-xs font-mono text-muted-foreground/70" },
              `PID: ${command.pid}`
            )
          : null,
      ]);
    },
  },
  {
    id: "actions",
    header: () => h("div", { class: "text-center" }, "Actions"),
    cell: ({ row }) => {
      return h("div", { class: "flex items-center justify-center gap-1" }, [
        h(
          Button,
          {
            variant: "ghost",
            size: "icon",
            class: "h-8 w-8 text-muted-foreground hover:text-primary",
            onClick: () => console.log("Start", row.original.id),
          },
          [h(PlayIcon, { class: "h-4 w-4" })]
        ),
        h(
          Button,
          {
            variant: "ghost",
            size: "icon",
            class: "h-8 w-8 text-muted-foreground hover:text-primary",
            onClick: () => console.log("Restart", row.original.id),
          },
          [h(RestartIcon, { class: "h-4 w-4" })]
        ),
        h(
          Button,
          {
            variant: "ghost",
            size: "icon",
            class: "h-8 w-8 text-muted-foreground",
            onClick: () => console.log("Menu", row.original.id),
          },
          [h(MenuDotsIcon, { class: "h-4 w-4" })]
        ),
      ]);
    },
  },
];

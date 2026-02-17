import type { ColumnDef } from "@tanstack/vue-table";
import { h } from "vue";
import { DirectoryIcon } from "@/assets/Icons.ts";
import CommandMenu from "@/components/tables/commands/CommandMenu.vue";
import type { ICommand } from "@/lib/api/api.types.ts";
import CommandTableTitle from "@/components/tables/commands/CommandTableTitle.vue";

const statusConfig = {
  running: { dot: "bg-emerald-500" },
  stopped: { dot: "bg-gray-500" },
  error: { dot: "bg-red-500" },
};

// demo data, will be finalized after execution module is done
export const columns: ColumnDef<ICommand>[] = [
  {
    accessorKey: "name",
    header: "Command Name",
    cell: ({ row }) => {
      return h(CommandTableTitle, {
        command: row.original,
      });
    },
  },
  {
    accessorKey: "status",
    header: "Status",
    cell: ({ row }) => {
      const status = "running";
      const style = statusConfig[status] ?? { dot: "bg-gray-500" };

      return h("div", { class: "flex flex-col gap-1" }, [
        h("div", { class: "flex items-center gap-2" }, [
          h("span", { class: `h-2 w-2 rounded-full ${style.dot}` }),
          h("span", { class: "text-sm text-xs mb-1 capitalize" }, status),
        ]),
        h(
          "div",
          {
            class: "flex items-center gap-1.5 text-xs text-muted-foreground",
          },
          [
            h(DirectoryIcon, { class: "h-3.5 w-3.5 shrink-0" }),
            h("span", { class: "truncate" }, row.original.workingDirectory),
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
          "command.activityTime"
        ),
        command.id
          ? h(
              "span",
              { class: "text-xs font-mono text-muted-foreground/70" },
              `PID: ${command.id}`
            )
          : null,
      ]);
    },
  },
  {
    id: "actions",
    header: () => h("div", { class: "text-center" }, "Actions"),
    cell: ({ row }) => {
      return h(CommandMenu, {
        id: row.original.id,
      });
    },
  },
];

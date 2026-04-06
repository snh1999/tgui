<script setup lang="ts">
  import { Copy, Play, Square, Trash2 } from "lucide-vue-next";
  import { computed, onMounted, ref } from "vue";
  import { useRoute } from "vue-router";
  import CategoryBadge from "@/components/categories/CategoryBadge.vue";
  import CommandDetails from "@/components/commands/CommandDetails.vue";
  import HistoryTable from "@/components/commands/HistoryTable.vue";
  import CreateCommandsDialog from "@/components/forms/commands/CreateCommandsDialog.vue";
  import UpdateCommandsDialog from "@/components/forms/commands/UpdateCommandsDialog.vue";
  import LogViewer from "@/components/logs/LogViewer.vue";
  import ToggleFavoriteButton from "@/components/shared/ToggleFavoriteButton.vue";
  import { Badge } from "@/components/ui/badge";
  import { Button } from "@/components/ui/button";
  import { ScrollArea } from "@/components/ui/scroll-area";
  import Loading from "@/components/ui/tgui/Loading.vue";
  import {
    useGetCommand,
    useToggleFavoriteCommand,
  } from "@/lib/api/composables/commands.ts";
  import { useGetCommandExecutionHistory } from "@/lib/api/composables/history.ts";
  import {
    useKillProcess,
    useSpawnCommand,
  } from "@/lib/api/composables/process.ts";
  import { loadExecutionLogs } from "@/lib/log-persistence.ts";
  import PageWrapper from "@/pages/PageWrapper.vue";
  import { useExecutionStore } from "@/stores/execution.store.ts";

  const route = useRoute();

  const commandId = computed(() => Number(route.params.id));

  const { data: command, isLoading } = useGetCommand(commandId);
  const { data: history } = useGetCommandExecutionHistory(commandId);
  const { mutate: toggleFavorite } = useToggleFavoriteCommand();

  const showDeleteConfirm = ref(false);
  const store = useExecutionStore();

  const { mutate: spawnCommand, isPending: isSpawning } = useSpawnCommand();
  const { mutate: killProcess, isPending: isKilling } = useKillProcess();

  const activeExecutionId = ref<number | null>(null);

  const isRunning = computed(() => store.isCommandRunning(commandId.value));

  const currentLogs = computed(() =>
    activeExecutionId.value
      ? store.getExecutionLogs(activeExecutionId.value)
      : []
  );

  const lastRunStatus = computed(() => {
    if (!activeExecutionId.value) {
      return null;
    }
    const exec = store.getExecution(activeExecutionId.value);
    if (!exec || exec.status === "running" || exec.status === "stopping") {
      return null;
    }
    return exec.status;
  });

  onMounted(() => {
    // 1. Same session — process might still be running or was recently stopped
    const inMemory = store.getLastExecutionForCommand(commandId.value);
    if (inMemory) {
      activeExecutionId.value = inMemory.id;
      return;
    }

    // 2. Previous session — restore from localStorage so logs survive app restart
    const persisted = loadExecutionLogs(commandId.value);
    if (persisted) {
      store.addExecution({
        id: persisted.executionId,
        commandId: persisted.commandId,
        status: persisted.status,
        startedAt: persisted.startedAt,
        completedAt: persisted.completedAt,
        triggeredBy: "manual",
        commandName: persisted.commandName,
        logs: persisted.logs,
        lastAccessedAt: new Date().toISOString(),
      });
      activeExecutionId.value = persisted.executionId;
    }
  });

  function handleRun() {
    spawnCommand(commandId.value, {
      onSuccess: (executionId) => {
        activeExecutionId.value = executionId;

        store.addExecution({
          id: executionId,
          commandId: commandId.value,
          commandName: command.value?.name ?? "",
          status: "running",
          triggeredBy: "manual",
          logs: [],
          startedAt: new Date().toISOString(),
          lastAccessedAt: new Date().toISOString(),
        });
      },
    });
  }

  function handleStop() {
    if (!activeExecutionId.value) {
      return;
    }
    killProcess(activeExecutionId.value);
  }
</script>

<template>
  <Loading v-if="isLoading" />
  <PageWrapper v-else-if="command">
    <div class="flex items-start gap-3">
      <div class="flex-1 min-w-0 space-y-1.5">
        <h2 class="text-base font-semibold leading-tight truncate">
          {{ command.name }}
        </h2>
        <div class="flex items-center gap-2 flex-wrap">
          <CategoryBadge
            v-if="command.categoryId"
            :categoryId="command.categoryId"
          />
          <Badge
            v-if="history && history[0]?.status"
            variant="outline"
            class="text-[10px] px-1.5 py-0 h-4"
          >
            Last: {{ history[0].status }}
          </Badge>
        </div>
      </div>

      <div class="flex items-center gap-2 shrink-0">
        <ToggleFavoriteButton
          v-if="command"
          :isFavorite="command.isFavorite"
          @toggleFavorite="toggleFavorite({ id: command.id })"
        />

        <!-- Last-run status badge shown when a previous execution is in view -->
        <Badge
          v-if="lastRunStatus"
          variant="outline"
          class="text-[10px] px-1.5 h-7 capitalize"
        >
          {{ lastRunStatus }}
        </Badge>

        <Button
          v-if="!isRunning"
          size="sm"
          class="h-7 px-2.5 text-xs gap-1.5"
          :disabled="isSpawning"
          @click="handleRun"
        >
          <Play class="h-3 w-3" />
          {{ isSpawning ? "Starting…" : "Run" }}
        </Button>

        <Button
          v-else
          size="sm"
          variant="destructive"
          class="h-7 px-2.5 text-xs gap-1.5"
          :disabled="isKilling"
          @click="handleStop"
        >
          <Square class="h-3 w-3" />
          Stop
        </Button>

        <UpdateCommandsDialog
          :id="command.id"
          triggerVariant="outline"
          viewTrigger
        />
        <CreateCommandsDialog :command="command">
          <Copy class="h-3.5 w-3.5" />
          Duplicate
        </CreateCommandsDialog>

        <Button
          variant="destructive"
          size="xs"
          class="border"
          @click="showDeleteConfirm = true"
        >
          <Trash2 class="h-3.5 w-3.5" />
          Delete
        </Button>
      </div>
    </div>

    <ScrollArea class="flex-1 min-h-0">
      <div class="py-4 space-y-5">
        <CommandDetails :command="command" :data="command" />
        <LogViewer :executionId="activeExecutionId" :logs="currentLogs" />
        <HistoryTable :commandId="commandId" />
      </div>
    </ScrollArea>
  </PageWrapper>
</template>

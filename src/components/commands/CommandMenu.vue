<script setup lang="ts">
  import { computed, ref } from "vue";
  import {
    CopyIcon,
    DeleteIcon,
    EditIcon,
    ExternalIcon,
    MenuDotsIcon,
    PlayIcon,
    ShutdownIcon,
    StopIcon,
  } from "@/assets/Icons.ts";
  import UpdateCommandsDialog from "@/components/forms/commands/UpdateCommandsDialog.vue";
  import { Button } from "@/components/ui/button";
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from "@/components/ui/dropdown-menu";
  import ConfirmDialog from "@/components/ui/tgui/ConfirmDialog.vue";
  import { useDeleteCommand } from "@/lib/api/composables/commands.ts";
  import { useRouter } from "vue-router";
  import { routePaths } from "@/router";
  import { Play } from "lucide-vue-next";
  import { TExecutionStatus } from "@/lib/api/api.types.ts";

  const props = defineProps<{
    id: number;
    isCompact?: boolean;
    status?: TExecutionStatus;
  }>();

  const router = useRouter();
  const { mutate: deleteCommand } = useDeleteCommand();

  const buttonStyle = "h-7 text-muted-foreground";

  const deleteDialogOpen = ref(false);
  const updateDialogOpen = ref(false);

  function goToCommandPage() {
    router.push(`${routePaths.commands}/${props.id}`);
  }

  function onDeleteClick() {
    deleteCommand(props.id);
    deleteDialogOpen.value = false;
  }

  const variant = computed(() => (props.isCompact ? "ghost" : "outline"));
  const size = computed(() => (props.isCompact ? "icon" : "xs"));
  const isRunning = computed(() => props.status === "running");
  const isStopping = computed(() => props.status === "stopping");
</script>

<template>
  <div class="flex items-center justify-center gap-1">
    <Button
      :variant="variant"
      :disabled="isStopping"
      :title="isStopping ? 'Stopping…' : 'Stop process'"
      :size="size"
      :class="buttonStyle"
    >
      <StopIcon v-if="isRunning" />
      <Play v-else class="h-3 w-3" />
      <span v-if="!isCompact && isRunning">Stop</span>
      <span v-if="!isCompact && !isRunning">
        {{ isStopping ? 'Stopping…' : 'Run' }}
      </span>
    </Button>

    <Button
      v-if="isRunning"
      title="View live logs"
      :variant="variant"
      size="xs"
      :class="buttonStyle"
    >
      Logs
    </Button>

    <DropdownMenu>
      <DropdownMenuTrigger as-child>
        <Button :variant="variant" size="icon-sm" :class="buttonStyle">
          <MenuDotsIcon />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent class="w-45" align="start">
        <DropdownMenuItem @select="goToCommandPage">
          <ExternalIcon />
          View Details
        </DropdownMenuItem>

        <DropdownMenuItem disabled>
          <CopyIcon />
          Duplicate
        </DropdownMenuItem>

        <DropdownMenuItem @select="updateDialogOpen = true">
          <EditIcon />
          Edit Command
        </DropdownMenuItem>

        <DropdownMenuItem variant="destructive" disabled>
          <ShutdownIcon />
          Force Kill
        </DropdownMenuItem>

        <DropdownMenuItem
          variant="destructive"
          @select="deleteDialogOpen = true"
        >
          <DeleteIcon />
          Delete Command
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>

    <UpdateCommandsDialog v-model:open="updateDialogOpen" :id="id" />

    <ConfirmDialog
      v-model:open="deleteDialogOpen"
      description="Are you sure you want to delete the command"
      :action="onDeleteClick"
    />
  </div>
</template>

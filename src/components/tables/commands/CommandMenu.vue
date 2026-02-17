<script setup lang="ts">
  import {
    DeleteIcon,
    EditIcon,
    ExternalIcon,
    MenuDotsIcon,
    PlayIcon,
    RestartIcon,
  } from "@/assets/Icons.ts";
  import { Button } from "@/components/ui/button";
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from "@/components/ui/dropdown-menu";
  import { useDeleteCommand } from "@/lib/api/composables/commands.ts";
  import { ref } from "vue";
  import ConfirmDialog from "@/components/ui/tgui/ConfirmDialog.vue";
  import UpdateCommandsDialog from "@/components/forms/commands/UpdateCommandsDialog.vue";

  const { id } = defineProps<{
    id: number;
  }>();

  const { mutate: deleteCommand } = useDeleteCommand();

  const buttonStyle = "h-8 w-8 text-muted-foreground hover:text-primary";

  const deleteDialogOpen = ref(false);
  const updateDialogOpen = ref(false);

  function onDeleteclick() {
    deleteCommand(id);
    deleteDialogOpen.value = false;
  }
</script>

<template>
  <div class="flex items-center justify-center gap-1">
    <Button variant="ghost" size="icon" :class="buttonStyle">
      <PlayIcon />
    </Button>
    <Button variant="ghost" size="icon" :class="buttonStyle">
      <RestartIcon />
    </Button>
    <DropdownMenu>
      <DropdownMenuTrigger as-child>
        <Button variant="ghost" size="icon" :class="buttonStyle">
          <MenuDotsIcon />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent class="w-45" align="start">
        <DropdownMenuItem>
          <ExternalIcon />
          View Details
        </DropdownMenuItem>

        <DropdownMenuItem @select="updateDialogOpen = true">
          <EditIcon />
          Edit
        </DropdownMenuItem>

        <DropdownMenuItem
          class="text-destructive"
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
      :action="onDeleteclick"
    />
  </div>
</template>

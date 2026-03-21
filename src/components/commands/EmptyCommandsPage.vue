<script setup lang="ts">
  import { FolderCode } from "lucide-vue-next";
  import CreateCommandsDialog from "@/components/forms/commands/CreateCommandsDialog.vue";
  import {
    Empty,
    EmptyContent,
    EmptyDescription,
    EmptyHeader,
    EmptyMedia,
    EmptyTitle,
  } from "@/components/ui/empty";
</script>

<template>
  <Empty class="h-screen text-muted-foreground">
    <EmptyHeader>
      <EmptyMedia variant="icon">
        <FolderCode />
      </EmptyMedia>
      <EmptyTitle class="text-muted-foreground">No Commands Yet</EmptyTitle>
      <EmptyDescription>
        You haven't created any commands yet. Get started by creating your first
        command.
      </EmptyDescription>
    </EmptyHeader>
    <EmptyContent>
      <div class="flex gap-2">
        <CreateCommandsDialog
          viewTrigger
          triggerSize="lg"
          triggerVariant="primary"
        />
      </div>
    </EmptyContent>
  </Empty>
</template>

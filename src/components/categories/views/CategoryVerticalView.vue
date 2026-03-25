<!--TODO consider possibility of merging with vertical view -->
<script setup lang="ts">
  import { ChevronDown } from "lucide-vue-next";
  import { GroupIcon, TerminalIcon } from "@/assets/Icons.ts";
  import { Button } from "@/components/ui/button";
  import CommandsDisplay from "@/components/commands/CommandsDisplay.vue";
  import GroupsDisplay from "@/components/groups/GroupsDisplay.vue";
  import { ICommand, IGroup } from "@/lib/api/api.types.ts";
  import { ref } from "vue";

  defineProps<{
    groups?: IGroup[];
    commands?: ICommand[];
    groupsCollapsed: boolean;
    commandsCollapsed: boolean;
  }>();

  const groupsCollapsed = ref(false);
  const commandsCollapsed = ref(false);
</script>

<template>
  <div class="flex flex-col flex-1 overflow-hidden">
    <div
      class="flex flex-col overflow-hidden transition-all"
      :class="groupsCollapsed ? 'shrink-0' : 'flex-1'"
    >
      <div
        class="sticky top-0 z-10 shrink-0 flex items-center justify-between px-4 py-2 border-b bg-muted/30"
      >
        <span
          class="flex items-center gap-2 text-sm font-medium text-muted-foreground"
        >
          <GroupIcon class="size-4" />
          Groups ({{ groups?.length ?? 0 }})
        </span>
        <Button
          variant="ghost"
          size="icon"
          class="size-6"
          @click="groupsCollapsed = !groupsCollapsed"
        >
          <ChevronDown
            class="size-4 transition-transform duration-200"
            :class="{ 'rotate-180': !groupsCollapsed }"
          />
        </Button>
      </div>
      <div v-if="!groupsCollapsed" class="flex-1 overflow-y-auto p-5">
        <GroupsDisplay v-if="groups" :groups="groups" view="grid" />
      </div>
    </div>

    <div class="h-px bg-border shrink-0" />

    <div
      class="flex flex-col overflow-hidden transition-all"
      :class="commandsCollapsed ? 'shrink-0' : 'flex-1'"
    >
      <div
        class="sticky top-0 z-10 shrink-0 flex items-center justify-between px-4 py-2 border-b bg-muted/30"
      >
        <span
          class="flex items-center gap-2 text-sm font-medium text-muted-foreground"
        >
          <TerminalIcon class="size-4" />
          Commands ({{ commands?.length ?? 0 }})
        </span>
        <Button
          variant="ghost"
          size="icon"
          class="size-6"
          @click="commandsCollapsed = !commandsCollapsed"
        >
          <ChevronDown
            class="size-4 transition-transform duration-200"
            :class="{ 'rotate-180': !commandsCollapsed }"
          />
        </Button>
      </div>
      <div v-if="!commandsCollapsed" class="flex-1 overflow-y-auto p-5">
        <CommandsDisplay v-if="commands" :commands="commands" view="grid" />
      </div>
    </div>
  </div>
</template>

<style scoped>
</style>

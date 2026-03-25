<script setup lang="ts">
  import { ChevronDown, ChevronUp } from "lucide-vue-next";
  import { GroupIcon, TerminalIcon } from "@/assets/Icons.ts";
  import { Button } from "@/components/ui/button";
  import CommandsDisplay from "@/components/commands/CommandsDisplay.vue";
  import GroupsDisplay from "@/components/groups/GroupsDisplay.vue";
  import { ICommand, IGroup } from "@/lib/api/api.types.ts";
  import { useCategoryStore } from "@/stores/categories.store.ts";

  defineProps<{
    groups?: IGroup[];
    commands?: ICommand[];
    groupsCollapsed: boolean;
    commandsCollapsed: boolean;
  }>();

  const store = useCategoryStore();
</script>

<template>
  <div class="flex flex-col flex-1 overflow-hidden">
    <div
      v-if="groupsCollapsed"
      class="shrink-0 flex items-center justify-between px-4 py-2 border-b bg-muted/30"
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
        @click="groupsCollapsed = false"
      >
        <ChevronDown class="size-4" />
      </Button>
    </div>
    <div
      v-if="!groupsCollapsed || !commandsCollapsed"
      class="flex flex-1 overflow-hidden"
    >
      <div
        v-if="!groupsCollapsed"
        class="flex flex-col overflow-hidden"
        :class="commandsCollapsed ? 'flex-1' : 'w-2/5 border-r'"
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
            @click="groupsCollapsed = true"
          >
            <ChevronUp class="size-4" />
          </Button>
        </div>
        <div class="flex-1 overflow-y-auto p-5">
          <GroupsDisplay
            v-if="groups"
            :groups="groups"
            :view="store.groupsView"
          />
        </div>
      </div>
      <div
        v-if="!commandsCollapsed"
        class="flex flex-col flex-1 overflow-hidden"
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
            @click="commandsCollapsed = true"
          >
            <ChevronUp class="size-4" />
          </Button>
        </div>
        <div class="flex-1 overflow-y-auto p-5">
          <CommandsDisplay
            v-if="commands"
            :commands="commands"
            :view="store.commandsView"
          />
        </div>
      </div>
    </div>

    <div
      v-if="commandsCollapsed"
      class="shrink-0 flex items-center justify-between px-4 py-2 border-t bg-muted/30"
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
        @click="commandsCollapsed = false"
      >
        <ChevronUp class="size-4" />
      </Button>
    </div>
  </div>
</template>

<style scoped>
</style>

<script setup lang="ts">
  import { ChevronDown, ChevronUp } from "lucide-vue-next";
  import { ref, watch } from "vue";
  import { GroupIcon, TerminalIcon } from "@/assets/Icons.ts";
  import CommandsDisplay from "@/components/commands/CommandsDisplay.vue";
  import GroupsDisplay from "@/components/groups/GroupsDisplay.vue";
  import { Button } from "@/components/ui/button";
  import type { ICommand, IGroup } from "@/lib/api/api.types.ts";
  import { useCategoryStore } from "@/stores/categories.store.ts";

  const props = defineProps<{
    layout: "horizontal" | "vertical";
    groups?: IGroup[];
    commands?: ICommand[];
  }>();

  const store = useCategoryStore();
  const groupsCollapsed = ref(false);
  const commandsCollapsed = ref(false);

  watch(
    () => props.groups?.length,
    (len) => {
      if (len === 0) {
        groupsCollapsed.value = true;
      }
    },
    { immediate: true }
  );
  watch(
    () => props.commands?.length,
    (len) => {
      if (len === 0) {
        commandsCollapsed.value = true;
      }
    },
    { immediate: true }
  );
</script>

<template>
  <template v-if="layout === 'horizontal'">
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

  <template v-else>
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
</template>

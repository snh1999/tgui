<script setup lang="ts">
  import {
    Tabs,
    TabsContent,
    TabsList,
    TabsTrigger,
  } from "@/components/ui/tabs";
  import { GroupIcon, TerminalIcon } from "@/assets/Icons.ts";
  import CommandsDisplay from "@/components/commands/CommandsDisplay.vue";
  import DataViewToggle from "@/components/views/DataViewToggle.vue";
  import GroupsDisplay from "@/components/groups/GroupsDisplay.vue";
  import { ref } from "vue";
  import { ICommand, IGroup } from "@/lib/api/api.types.ts";
  import { useCategoryStore } from "@/stores/categories.store.ts";

  defineProps<{
    groups?: IGroup[];
    commands?: ICommand[];
  }>();

  const store = useCategoryStore();

  const activeTab = ref("commands");
</script>

<template>
  <Tabs v-model="activeTab" class="flex flex-col flex-1 overflow-hidden">
    <div
      class="shrink-0 flex items-center justify-between p-3 border-b bg-muted/30"
    >
      <TabsList>
        <TabsTrigger value="commands" class="gap-2">
          <TerminalIcon class="size-4" />
          Commands
        </TabsTrigger>
        <TabsTrigger value="groups" class="gap-2">
          <GroupIcon class="size-4" />
          Groups
        </TabsTrigger>
      </TabsList>

      <DataViewToggle
        v-if="activeTab === 'commands'"
        v-model:view="store.commandsView"
      />
      <DataViewToggle v-else v-model:view="store.groupsView" />
    </div>

    <TabsContent value="commands" class="flex-1 overflow-y-auto p-4">
      <CommandsDisplay
        v-if="commands"
        :commands="commands"
        :view="store.commandsView"
      />
    </TabsContent>

    <TabsContent value="groups" class="flex-1 overflow-y-auto p-4">
      <GroupsDisplay v-if="groups" :groups="groups" :view="store.groupsView" />
    </TabsContent>
  </Tabs>
</template>

<style scoped>
</style>

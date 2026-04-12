<script setup lang="ts">
  import { LucideCirclePlus } from "lucide-vue-next";
  import { ref } from "vue";
  import CreateCategoryDialog from "@/components/forms/categories/CreateCategoryDialog.vue";
  import CreateCommandsDialog from "@/components/forms/commands/CreateCommandsDialog.vue";
  import CreateGroupDialog from "@/components/forms/groups/CreateGroupDialog.vue";
  import { Button } from "@/components/ui/button";
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from "@/components/ui/dropdown-menu";
  import { createCommandHotKeys } from "@/lib/composables/createCommandHotkey.ts";

  const createCommandOpen = ref(false);
  const createGroupOpen = ref(false);
  const createCategoryOpen = ref(false);
  const { clipboardText } = createCommandHotKeys(createCommandOpen);
</script>

<template>
  <DropdownMenu>
    <DropdownMenuTrigger>
      <Button variant="ghost" size="icon-sm">
        <LucideCirclePlus />
      </Button>
    </DropdownMenuTrigger>
    <DropdownMenuContent>
      <DropdownMenuItem @select="createCommandOpen = true">
        Command
      </DropdownMenuItem>
      <DropdownMenuItem @select="createGroupOpen = true">
        Group
      </DropdownMenuItem>
      <DropdownMenuItem @select="createCategoryOpen = true">
        Category
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>

  <CreateCommandsDialog
    v-model:open="createCommandOpen"
    :commandText="clipboardText"
  />
  <CreateGroupDialog v-model:open="createGroupOpen" />
  <CreateCategoryDialog v-model:open="createCategoryOpen" />
</template>

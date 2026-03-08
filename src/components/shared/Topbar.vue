<script setup lang="ts">
  import { ref } from "vue";
  import { SearchIcon, SettingsIcon } from "@/assets/Icons.ts";
  import CreateCommandsDialog from "@/components/forms/commands/CreateCommandsDialog.vue";
  import { Button } from "@/components/ui/button";
  import {
    InputGroup,
    InputGroupAddon,
    InputGroupInput,
  } from "@/components/ui/input-group";
  import { SidebarTrigger } from "@/components/ui/sidebar";

  const resultCount = ref("");
  const createDialogOpen = ref(false);
</script>

<template>
  <header class="flex bg-sidebar px-8 py-4 gap-5">
    <div class="md:hidden">
      <SidebarTrigger />
    </div>
    <InputGroup>
      <InputGroupInput placeholder="Search..." />
      <InputGroupAddon>
        <SearchIcon />
      </InputGroupAddon>
      <InputGroupAddon align="inline-end">{{ resultCount }}</InputGroupAddon>
    </InputGroup>

    <Button variant="outline">
      <SettingsIcon />
      Settings
    </Button>

    <CreateCommandsDialog v-model:open="createDialogOpen" view-trigger />
  </header>
</template>

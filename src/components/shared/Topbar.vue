<script setup lang="ts">
  import { SearchIcon, SettingsIcon } from "@/assets/Icons.ts";
  import {
    InputGroup,
    InputGroupAddon,
    InputGroupInput,
  } from "@/components/ui/input-group";
  import { ref } from "vue";
  import { Button } from "@/components/ui/button";
  import { SidebarTrigger } from "@/components/ui/sidebar";
  import CreateCommandsDialog from "@/components/forms/commands/CreateCommandsDialog.vue";

  const resultCount = ref("");
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

    <CreateCommandsDialog />
  </header>
</template>

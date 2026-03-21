<script setup lang="ts">
  import { Terminal } from "lucide-vue-next";
  import {
    BashIcon,
    FishShellIcon,
    NuShellIcon,
    PowershellIcon,
    ZshIcon,
  } from "@/assets/Icons.ts";
  import { ICommand } from "@/lib/api/api.types.ts";

  defineProps<{
    command: ICommand;
  }>();
</script>

<template>
  <code
    class="flex items-center gap-2 px-3 py-2 mb-1 rounded bg-muted/80 font-mono text-xs"
  >
    <span v-if="command.shell" class="italic">({{ command.shell }})</span>
    <BashIcon v-if="command.shell === 'bash'" />
    <ZshIcon v-if="command.shell === 'zsh'" />
    <FishShellIcon v-if="command.shell === 'fish'" />
    <NuShellIcon v-if="command.shell === 'nu'" />
    <PowershellIcon v-if="command.shell === 'powershell'" />
    <Terminal v-else class="h-3 w-3 shrink-0" />

    <span class="truncate"
      >{{ command.command }}
      {{ command.arguments?.length ? ' ' + command.arguments?.join(' ') : '' }}</span
    >
  </code>
</template>

<style scoped>
</style>

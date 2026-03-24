<script setup lang="ts">
  import { ref } from "vue";
  import { CrossIcon, SearchIcon } from "@/assets/Icons.ts";
  import { Button } from "@/components/ui/button";
  import {
    InputGroup,
    InputGroupAddon,
    InputGroupInput,
  } from "@/components/ui/input-group";
  import { useCommandsViewStore } from "@/stores/commands.store.ts";

  const resultCount = ref("");
  const commandsView = useCommandsViewStore();
</script>

<template>
  <InputGroup
    class="w-60 md:w-80 focus-within:w-80 focus-within:md:w-110 transition-[width] duration-200"
  >
    <InputGroupInput
      v-model="commandsView.search"
      placeholder="Search by name, command, description..."
    />
    <InputGroupAddon>
      <SearchIcon />
    </InputGroupAddon>
    <InputGroupAddon v-if="commandsView.search" align="inline-end">
      <Button variant="ghost" size="icon-sm" @click="commandsView.search = ''">
        <CrossIcon />
      </Button>
    </InputGroupAddon>
    <InputGroupAddon v-if="resultCount" align="inline-end">
      {{ resultCount }}
    </InputGroupAddon>
  </InputGroup>
</template>

<style scoped>
</style>

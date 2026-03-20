<script setup lang="ts">
  import { useColorMode } from "@vueuse/core";
  import { MoonIcon, SunIcon } from "@/assets/Icons.ts";
  import { Button } from "@/components/ui/button";

  const mode = useColorMode({
    disableTransition: false,
  });
</script>

<template>
  <Button variant="ghost" @click="mode = 'dark'" v-if="mode === 'light'">
    <MoonIcon />
  </Button>
  <Button variant="ghost" @click="mode = 'light'" v-if="mode === 'dark'">
    <SunIcon />
  </Button>
</template>

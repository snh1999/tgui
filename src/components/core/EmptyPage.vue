<script setup lang="ts">
  import {
    Empty,
    EmptyContent,
    EmptyDescription,
    EmptyHeader,
    EmptyTitle,
  } from "@/components/ui/empty";

  defineProps<{
    title: string;
    description: string;
  }>();
</script>

<template>
  <Empty class="h-screen text-muted-foreground">
    <EmptyHeader>
      <EmptyTitle class="text-muted-foreground">{{ title }}</EmptyTitle>
      <EmptyDescription>{{ description }}</EmptyDescription>
    </EmptyHeader>
    <EmptyContent>
      <slot />
    </EmptyContent>
  </Empty>
</template>

<script setup lang="ts">
  import type { Component } from "vue";
  import {
    Item,
    ItemActions,
    ItemContent,
    ItemFooter,
    ItemMedia,
    ItemTitle,
  } from "@/components/ui/item";
  import { Badge } from "@/components/ui/badge";

  defineProps<{
    icon: Component;
    color?: "green" | "blue" | "red" | "gray";
    badgeText?: string;
    statValue: string;
    label: string;
    highlightCard?: boolean;
  }>();
</script>

<template>
  <Item variant="card" class="stat-card">
    <ItemMedia
      class="h-10 w-10 rounded-lg"
      :class="!highlightCard ? color : ''"
    >
      <Component :is="icon" />
    </ItemMedia>
    <ItemContent>
      <ItemTitle class="text-2xl font-bold">{{ statValue }}</ItemTitle>
    </ItemContent>

    <ItemActions>
      <Badge v-if="badgeText">{{ badgeText }}</Badge>
    </ItemActions>

    <ItemFooter>{{ label }}</ItemFooter>
  </Item>
</template>

<style scoped>
  .stat-card {
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    border: 1px solid var(--color-border);
    max-width: 425px;
  }

  .green {
    background: color-mix(in srgb, var(--color-primary) 15%, transparent);
    color: var(--color-primary);
  }

  .blue {
    background: color-mix(in srgb, var(--color-secondary) 15%, transparent);
    color: var(--color-secondary);
  }

  .red {
    background: color-mix(in srgb, var(--color-danger) 15%, transparent);
    color: var(--color-danger);
  }

  .gray {
    background: var(--color-bg);
    color: var(--color-text-secondary);
  }
</style>

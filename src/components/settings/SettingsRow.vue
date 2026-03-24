<script setup lang="ts">
  defineProps<{
    label: string;
    description?: string;
  }>();
</script>

<template>
  <div class="setting-row">
    <div class="setting-info">
      <p class="setting-label">{{ label }}</p>
      <p v-if="description" class="setting-description">{{ description }}</p>
    </div>
    <div class="setting-control">
      <slot />
    </div>
  </div>
</template>

<style scoped>
  .setting-row {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 1.5rem;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--border);
    transition: background-color 0.15s ease;
  }

  .setting-row:last-child {
    border-bottom: none;
  }

  .setting-row:hover {
    background: var(--muted);
  }

  .setting-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .setting-label {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--foreground);
    margin: 0;
    line-height: 1.4;
  }

  .setting-description {
    font-size: 0.75rem;
    color: var(--muted-foreground);
    margin: 0;
    line-height: 1.4;
  }

  .setting-control {
    flex-shrink: 0;
    display: flex;
    align-items: flex-start;
    padding-top: 0.125rem;
  }
</style>

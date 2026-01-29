<script setup lang="ts">
import type {Component} from 'vue'

defineProps<{
  icon: Component,
  color?: "green" | "blue" | "red" | "gray"
  badgeText?: string
  statValue: string
  label: string
  highlightCard?: boolean
}>()
</script>

<template>
  <div class="stat-card" :class="highlightCard ? color : ''">
    <div class="stat-header">
      <div class="stat-icon" :class="!highlightCard ? color : ''">
        <Component :is="icon"/>
      </div>
      <div class="stat-value">{{ statValue }}</div>

      <div class="stat-badge" v-if="badgeText">{{ badgeText }}</div>
    </div>
    <div>
      <div class="stat-label">{{ label }}</div>
    </div>

  </div>

</template>

<style scoped>
.stat-card {
  background: var(--color-surface);
  padding: var(--space-md) var(--space-lg);
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  border: 1px solid var(--color-border);
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


.stat-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 5px;
}

.stat-icon {
  width: 40px;
  height: 40px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.stat-badge {
  margin-left: auto;
  background: #d1fae5;
  color: #10b981;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
}

.stat-value {
  font-size: 28px;
  font-weight: 700;
  margin-bottom: 4px;
}

.stat-label {
  color: #666;
  font-size: 14px;
}

@container root (max-width: 500px) {
  .stat-card {
    padding: var(--space-sm) var(--space-md);
  }

  .stat-value {
    font-size: 20px;
  }

  .stat-icon {
    width: 32px;
    height: 32px;
  }
}

</style>

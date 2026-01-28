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
  background: white;
  padding: 10px 20px;
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
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

.green {
  background: #d1fae5;
  color: #10b981;
}

.blue {
  background: #dbeafe;
  color: #3b82f6;
}

.red {
  background: #fee2e2;
  color: #ef4444;
}

.gray {
  background: #f3f4f6;
  color: #6b7280;
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
</style>

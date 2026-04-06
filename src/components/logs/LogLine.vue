<script setup lang="ts">

defineProps<{
  timestamp: string
  type: string
  message: string
}>()
</script>

<template>
  <div class="log-line">
    <span class="log-time">{{ timestamp }}</span>
    <span class="log-type" :class="type">[{{ type }}]</span>
    <span>{{ message }}</span>
  </div>
</template>

<style scoped>
.log-line {
  margin-bottom: 4px;
  line-height: 1.6;
}

.log-time {
  color: #6b7280;
}

.log-type {
  font-weight: 600;
  margin: 0 8px;
  text-transform: uppercase;
}

.info {
  color: #3b82f6;
}

.log {
  color: #10b981;
}

.error {
  color: #ef4444;
}
</style>

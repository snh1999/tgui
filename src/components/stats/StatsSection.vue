<script setup lang="ts">

import StatCard from "@/components/stats/StatCard.vue";
import {ClockIcon, ErrorIcon, ListIcon, PlayIcon} from "@/assets/Icons.ts";
</script>

<template>
  <div class="stats">
    <StatCard
        :icon="ListIcon"
        color="green"
        badgeText="+2 today"
        statValue="12"
        label="Total Commands"
    />

    <StatCard
        :icon="PlayIcon"
        color="blue"
        statValue="3"
        label="Running Now"
    />

    <StatCard
        :icon="ErrorIcon"
        color="red"
        statValue="1"
        label="Errors Detected"
    />

    <StatCard
        :icon="ClockIcon"
        color="gray"
        statValue="4.2h"
        label="Total Uptime (24h)"
    />

  </div>

</template>

<style scoped>
.stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 20px;
  padding: 30px;
}
</style>

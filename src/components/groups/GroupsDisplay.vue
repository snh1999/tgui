<script setup lang="ts">
  import DataDisplay from "@/components/views/DataDisplay.vue";
  import { IGroup } from "@/lib/api/api.types.ts";
  import { TViewMode } from "@/stores/commands.store.ts";
  import GroupCard from "@/components/groups/GroupCard.vue";

  defineProps<{
    groups: IGroup[];
    view: TViewMode;
    isCard?: boolean;
  }>();
</script>

<template>
  <DataDisplay :view="view">
    <template #list>
      <GroupCard v-for="group in groups" :key="group.id" :group="group" />
    </template>

    <template #grid>
      <GroupCard
        v-for="group in groups"
        :key="group.id"
        :group="group"
        :isCard="true"
      />
    </template>

    <template #table>
      <!-- Groups table not implemented yet -->
    </template>
  </DataDisplay>
</template>

<script setup lang="ts">
  import {
    Card,
    CardAction,
    CardContent,
    CardFooter,
    CardHeader,
    CardTitle,
  } from "@/components/ui/card";
  import { ref } from "vue";
  import { IGroup } from "@/lib/api/api.types.ts";
  import UpdateGroupDialog from "@/components/forms/groups/UpdateGroupDialog.vue";

  const props = defineProps<{
    group: IGroup;
    isCard?: boolean;
  }>();

  const hovered = ref(false);
</script>

<template>
  <Card
    class="w-full mb-4 py-3 gap-1"
    @mouseenter="hovered = true"
    @mouseleave="hovered = false"
  >
    <CardHeader class="px-4">
      <CardTitle>{{ group.name }}</CardTitle>
      <CardAction v-if="!isCard" v-show="hovered"></CardAction>
    </CardHeader>
    <CardContent class=" pb-2">
      <blockquote class="mt-2 border-l-2 pl-2 italic text-xs truncate">
        {{ group.description ?? 'No description provided' }}
      </blockquote>
    </CardContent>
    <CardFooter v-if="isCard">
      <UpdateGroupDialog :id="group.id" viewTrigger />
    </CardFooter>
  </Card>
</template>

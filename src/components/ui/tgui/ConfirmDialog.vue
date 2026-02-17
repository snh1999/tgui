<script setup lang="ts">
  import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
  } from "@/components/ui/alert-dialog";
  import { computed } from "vue";

  const { variant } = defineProps<{
    title?: string;
    description: string;
    actionText?: string;
    variant?: "primary" | "secondary" | "success" | "warning" | "destructive";
    action: () => void;
  }>();
  const buttonStyle = computed(() => `bg-${variant ?? "primary"}`);

  const open = defineModel<boolean>("open");
</script>

<template>
  <AlertDialog v-model:open="open">
    <AlertDialogContent>
      <AlertDialogHeader>
        <AlertDialogTitle v-if="title">{{ title }}</AlertDialogTitle>
        <AlertDialogDescription>{{ description }}</AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel>Cancel</AlertDialogCancel>
        <AlertDialogAction :class="buttonStyle" @click="action">
          {{ actionText ?? "Ok" }}
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
</template>

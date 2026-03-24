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
    AlertDialogTrigger,
  } from "@/components/ui/alert-dialog";
  import { Button } from "@/components/ui/button";

  defineProps<{
    title?: string;
    description: string;
    actionText?: string;
  }>();

  const emit = defineEmits<{
    confirm: [];
  }>();

  const open = defineModel<boolean>("open");
</script>

<template>
  <AlertDialog v-model:open="open">
    <AlertDialogTrigger as-child v-if="$slots.default">
      <slot />
    </AlertDialogTrigger>
    <AlertDialogContent>
      <AlertDialogHeader>
        <AlertDialogTitle v-if="title">{{ title }}</AlertDialogTitle>
        <AlertDialogDescription>{{ description }}</AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel>Cancel</AlertDialogCancel>
        <Button
          asChild
          variant="destructive"
          class="border hover:text-destructive"
          @click="emit('confirm')"
        >
          <AlertDialogAction>{{ actionText ?? "Ok" }}</AlertDialogAction>
        </Button>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
</template>

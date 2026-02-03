<script setup lang="ts">
  import { useMediaQuery } from "@vueuse/core";
  import { Button } from "@/components/ui/button";
  import {
    Dialog,
    DialogClose,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from "@/components/ui/dialog";

  const isDesktop = useMediaQuery("(min-width: 640px)");

  defineProps<{
    open?: boolean;
    title?: string;
  }>();

  defineEmits<{
    "update:open": [value: boolean];
  }>();

  const isOpen = defineModel<boolean>("open", { default: false });
</script>

<template>
  <Dialog v-model:open="isOpen">
    <DialogContent
      class="sm:max-w-md"
      :class="{ 'px-2 pb-8 *:px-4': !isDesktop }"
    >
      <DialogHeader>
        <DialogTitle>{{ title || 'Dialog Title' }}</DialogTitle>

        <DialogDescription v-if="$slots.description">
          <slot name="description" />
        </DialogDescription>
      </DialogHeader>

      <div class="px-4 py-4">
        <slot>
          <p class="text-sm text-muted-foreground">No content provided.</p>
        </slot>
      </div>

      <DialogFooter class="flex gap-2 justify-end">
        <slot name="footer">
          <DialogClose as-child>
            <Button variant="outline">Close</Button>
          </DialogClose>
        </slot>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>

<script setup lang="ts">
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
  import { ScrollArea } from "@/components/ui/scroll-area";

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
      class="w-full max-w-lg xs:max-w-md sm:max-w-lg md:max-w-2xl lg:max-w-3xl xl:max-w-4xl"
    >
      <DialogHeader>
        <DialogTitle>{{ title || 'Dialog Title' }}</DialogTitle>

        <DialogDescription v-if="$slots.description">
          <slot name="description" />
        </DialogDescription>
      </DialogHeader>

      <ScrollArea class="max-h-[80vh] px-4 py-4">
        <div class="px-4 py-4">
          <slot>
            <p class="text-sm text-muted-foreground">No content provided.</p>
          </slot>
        </div>
      </ScrollArea>

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

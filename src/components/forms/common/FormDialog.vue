<script setup lang="ts">
  import { Button, type ButtonVariants } from "@/components/ui/button";
  import { Field } from "@/components/ui/field";
  import OpenDialog from "@/components/ui/tgui/OpenDialog.vue";

  const props = defineProps<{
    title: string;
    triggerVariant?: ButtonVariants["variant"];
    triggerSize?: ButtonVariants["size"];
  }>();

  const open = defineModel<boolean>("open", { default: false });
  const openDialog = () => {
    open.value = true;
  };
  const closeDialog = () => {
    open.value = false;
  };
</script>

<template>
  <Button
    v-if="$slots.trigger"
    @click="openDialog"
    :variant="triggerVariant"
    :size="triggerSize ?? 'xs'"
    class="gap-1"
  >
    <slot name="trigger" />
  </Button>

  <OpenDialog class="min-w-[50%]" v-model:open="open" :title="title">
    <slot :closeDialog="closeDialog" />

    <template #footer>
      <Field orientation="horizontal">
        <slot name="reset" />
      </Field>

      <Button variant="outline" @click="closeDialog">Cancel</Button>

      <slot name="submit" />
    </template>
  </OpenDialog>
</template>

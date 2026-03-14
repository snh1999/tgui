<script setup lang="ts">
  import { Button } from "@/components/ui/button";
  import { Field } from "@/components/ui/field";
  import OpenDialog from "@/components/ui/tgui/OpenDialog.vue";

  const props = defineProps<{
    title: string;
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
  <Button v-if="$slots.trigger" @click="openDialog" size="sm" class="gap-1">
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

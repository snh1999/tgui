<script setup lang="ts">
  import { ref } from "vue";
  import { COMMAND_FORM_ID } from "@/app.constants.ts";
  import { AddIcon } from "@/assets/Icons";
  import UpsertCommandForm from "@/components/forms/commands/UpsertCommandForm.vue";
  import { Button } from "@/components/ui/button";
  import FormDialog from "@/components/forms/common/FormDialog.vue";

  const props = defineProps<{
    viewTrigger?: boolean;
  }>();

  const createCommandFormRef = ref<InstanceType<
    typeof UpsertCommandForm
  > | null>(null);
</script>

<template>
  <FormDialog title="Create New Command">
    <template #trigger>
      <AddIcon />
      New Command
    </template>

    <template #default="{closeDialog}">
      <UpsertCommandForm @success="closeDialog" ref="createCommandFormRef" />
    </template>

    <template #reset>
      <Button
        type="button"
        variant="outline"
        @click="createCommandFormRef?.resetForm()"
        :isPending="createCommandFormRef?.isPending"
        :disabled="!createCommandFormRef?.isDirty"
      >
        Reset
      </Button>
    </template>

    <template #submit>
      <Button
        type="submit"
        :form="COMMAND_FORM_ID"
        :isPending="createCommandFormRef?.isPending"
        :disabled="!createCommandFormRef?.isValid"
      >
        Create
      </Button>
    </template>
  </FormDialog>
</template>

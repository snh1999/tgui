<script setup lang="ts">
  import { ref } from "vue";
  import { COMMAND_FORM_ID } from "@/app.constants.ts";
  import { AddIcon } from "@/assets/Icons";
  import UpsertCommandForm from "@/components/forms/commands/UpsertCommandForm.vue";
  import FormDialog from "@/components/forms/common/FormDialog.vue";
  import { Button } from "@/components/ui/button";
  import { ICommand } from "@/lib/api/api.types.ts";

  const props = defineProps<{
    viewTrigger?: boolean;
    command?: ICommand;
  }>();

  const createCommandFormRef = ref<InstanceType<
    typeof UpsertCommandForm
  > | null>(null);

  console.log(props.command);
</script>

<template>
  <FormDialog title="Create New Command">
    <template v-if="$slots.default" #trigger>
      <slot />
    </template>
    <template v-else-if="viewTrigger" #trigger>
      <AddIcon />
      New Command
    </template>

    <template #default="{closeDialog}">
      <UpsertCommandForm
        @success="closeDialog"
        ref="createCommandFormRef"
        :command="command"
        isCreate
      />
    </template>

    <template #reset>
      <Button
        type="button"
        variant="destructive"
        @click="createCommandFormRef?.resetForm()"
        :is-pending="createCommandFormRef?.isPending"
        :disabled="!createCommandFormRef?.isDirty"
      >
        Reset
      </Button>
    </template>

    <template #submit>
      <Button
        type="submit"
        variant="primary"
        :form="COMMAND_FORM_ID"
        :is-pending="createCommandFormRef?.isPending"
        :disabled="!createCommandFormRef?.isValid"
      >
        Create
      </Button>
    </template>
  </FormDialog>
</template>

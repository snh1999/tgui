<script setup lang="ts">
  import { ref } from "vue";
  import { COMMAND_FORM_ID } from "@/app.constants.ts";
  import UpsertCommandForm from "@/components/forms/commands/UpsertCommandForm.vue";
  import ErrorDisplay from "@/components/ui/tgui/ErrorDisplay.vue";
  import Loading from "@/components/ui/tgui/Loading.vue";
  import { useGetCommand } from "@/lib/api/composables/commands.ts";
  import FormDialog from "@/components/forms/common/FormDialog.vue";
  import { Button } from "@/components/ui/button";
  import { EditIcon } from "@/assets/Icons";

  const props = defineProps<{
    id: number;
    viewTrigger?: boolean;
  }>();

  const {
    data: command,
    isPending,
    isError,
    error,
    refetch,
  } = useGetCommand(props.id);

  const updateCommandFormRef = ref<InstanceType<
    typeof UpsertCommandForm
  > | null>(null);
</script>

<template>
  <FormDialog title="Update command">
    <template v-if="viewTrigger" #trigger>
      <EditIcon /> Edit
    </template>

    <template #default="{ closeDialog }">
      <Loading v-if="isPending" />
      <ErrorDisplay v-if="isError" :error="error" :retry="refetch" />
      <UpsertCommandForm
        v-if="command"
        :key="command.id"
        :command="command"
        @success="closeDialog"
        ref="updateCommandFormRef"
      />
    </template>

    <template #reset>
      <Button
        type="button"
        variant="outline"
        @click="updateCommandFormRef?.resetForm()"
        :isPending="updateCommandFormRef?.isPending"
        :disabled="!updateCommandFormRef?.isDirty"
      >
        Reset
      </Button>
    </template>

    <template #submit>
      <Button
        type="submit"
        :form="COMMAND_FORM_ID"
        :isPending="updateCommandFormRef?.isPending"
        :disabled="!updateCommandFormRef?.isValid ||!updateCommandFormRef?.isDirty"
      >
        Update
      </Button>
    </template>
  </FormDialog>
</template>

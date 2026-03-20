<script setup lang="ts">
  import { ref } from "vue";
  import { COMMAND_FORM_ID } from "@/app.constants.ts";
  import UpsertCommandForm from "@/components/forms/commands/UpsertCommandForm.vue";
  import ErrorDisplay from "@/components/ui/tgui/ErrorDisplay.vue";
  import Loading from "@/components/ui/tgui/Loading.vue";
  import { useGetCommand } from "@/lib/api/composables/commands.ts";
  import FormDialog from "@/components/forms/common/FormDialog.vue";
  import { Button } from "@/components/ui/button";
  import { Pencil } from "lucide-vue-next";

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
    <template v-if="$slots.default" #trigger>
      <slot />
    </template>
    <template v-else-if="viewTrigger" #trigger>
      <Pencil class="h-3.5 w-3.5" />
      Edit
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
        :is-pending="updateCommandFormRef?.isPending"
        :disabled="!updateCommandFormRef?.isDirty"
      >
        Reset
      </Button>
    </template>

    <template #submit>
      <Button
        type="submit"
        :form="COMMAND_FORM_ID"
        :is-pending="updateCommandFormRef?.isPending"
        :disabled="!updateCommandFormRef?.isValid || !updateCommandFormRef?.isDirty"
      >
        Update
      </Button>
    </template>
  </FormDialog>
</template>

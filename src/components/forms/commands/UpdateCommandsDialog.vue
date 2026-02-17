<script setup lang="ts">
  import { ref } from "vue";
  import { COMMAND_FORM_ID } from "@/app.constants.ts";
  import { EditIcon } from "@/assets/Icons";
  import UpsertCommandForm from "@/components/forms/commands/UpsertCommandForm.vue";
  import { Button } from "@/components/ui/button";
  import { Field } from "@/components/ui/field";
  import OpenDialog from "@/components/ui/tgui/OpenDialog.vue";
  import { Spinner } from "@/components/ui/spinner";
  import { useGetCommand } from "@/lib/api/composables/commands.ts";
  import Loading from "@/components/ui/tgui/Loading.vue";
  import ErrorDisplay from "@/components/ui/tgui/ErrorDisplay.vue";

  const open = defineModel<boolean>("open", { default: false });
  const openDialog = () => {
    open.value = true;
  };
  const closeDialog = () => {
    open.value = false;
  };

  const updateCommandFormRef = ref<InstanceType<
    typeof UpsertCommandForm
  > | null>(null);

  const prop = defineProps<{
    id: number;
    viewTrigger?: boolean;
  }>();

  const {
    data: command,
    isPending,
    isError,
    error,
    refetch,
  } = useGetCommand(prop.id);
</script>

<template>
  <header class="header">
    <Button v-if="viewTrigger" @click="openDialog" class="btn-primary gap-2">
      <EditIcon />
      Edit Command
    </Button>

    <OpenDialog class="min-w-[50%]" v-model:open="open" title="Update Command">
      <Loading v-if="isPending" />
      <ErrorDisplay v-if="isError" :error="error" :retry="refetch" />

      <UpsertCommandForm
        v-if="command"
        :key="command.id"
        :command="command"
        :onSuccess="closeDialog"
        ref="updateCommandFormRef"
      />

      <template #footer>
        <Field orientation="horizontal">
          <Button
            type="button"
            variant="outline"
            @click="updateCommandFormRef?.resetForm()"
            :isPending="updateCommandFormRef?.isPending"
          >
            Reset
            <Spinner v-if="updateCommandFormRef?.isPending" />
          </Button>
        </Field>

        <Button variant="outline" @click="closeDialog">Cancel</Button>
        <Button
          type="submit"
          :form="COMMAND_FORM_ID"
          :isPending="updateCommandFormRef?.isPending"
        >
          Update
        </Button>
      </template>
    </OpenDialog>
  </header>
</template>

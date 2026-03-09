<script setup lang="ts">
  import { ref } from "vue";
  import { GROUP_FORM_ID } from "@/app.constants.ts";
  import { AddIcon } from "@/assets/Icons";
  import { Button } from "@/components/ui/button";
  import UpsertGroupForm from "@/components/forms/groups/UpsertGroupForm.vue";
  import FormDialog from "@/components/forms/common/FormDialog.vue";

  const props = defineProps<{
    viewTrigger?: boolean;
  }>();

  const createGroupFormRef = ref<InstanceType<typeof UpsertGroupForm> | null>(
    null
  );
</script>

<template>
  <FormDialog title="Create New Group">
    <template v-if="viewTrigger" #trigger>
      <AddIcon />
      New Group
    </template>

    <template #default="{closeDialog}">
      <UpsertGroupForm @success="closeDialog" ref="createGroupFormRef" />
    </template>

    <template #reset>
      <Button
        variant="outline"
        @click="createGroupFormRef?.resetForm()"
        :is-pending="createGroupFormRef?.isPending"
        :disabled="!createGroupFormRef?.isDirty"
      >
        Reset
      </Button>
    </template>

    <template #submit>
      <Button
        type="submit"
        :form="GROUP_FORM_ID"
        :is-pending="createGroupFormRef?.isPending"
        :disabled="!createGroupFormRef?.isValid"
      >
        Create
      </Button>
    </template>
  </FormDialog>
</template>

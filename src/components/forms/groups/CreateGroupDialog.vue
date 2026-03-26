<script setup lang="ts">
  import { ref } from "vue";
  import { GROUP_FORM_ID } from "@/app.constants.ts";
  import { AddIcon } from "@/assets/Icons";
  import { Button } from "@/components/ui/button";
  import UpsertGroupForm from "@/components/forms/groups/UpsertGroupForm.vue";
  import FormDialog from "@/components/forms/common/FormDialog.vue";
  import { IGroup } from "@/lib/api/api.types.ts";

  const props = defineProps<{
    viewTrigger?: boolean;
    group?: IGroup;
  }>();

  const createGroupFormRef = ref<InstanceType<typeof UpsertGroupForm> | null>(
    null
  );
</script>

<template>
  <FormDialog title="Create New Group">
    <template v-if="$slots.default" #trigger>
      <slot />
    </template>
    <template v-if="viewTrigger" #trigger>
      <AddIcon />
      New Group
    </template>

    <template #default="{closeDialog}">
      <UpsertGroupForm
        @success="closeDialog"
        ref="createGroupFormRef"
        :group="group"
        isCreate
      />
    </template>

    <template #reset>
      <Button
        variant="destructive"
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
        variant="primary"
        :form="GROUP_FORM_ID"
        :is-pending="createGroupFormRef?.isPending"
        :disabled="!createGroupFormRef?.isValid"
      >
        Create
      </Button>
    </template>
  </FormDialog>
</template>

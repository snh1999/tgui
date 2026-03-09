<script setup lang="ts">
  import UpsertCategoryForm from "@/components/forms/categories/UpsertCategoryForm.vue";
  import { ref } from "vue";
  import FormDialog from "@/components/forms/common/FormDialog.vue";
  import { AddIcon } from "@/assets/Icons";
  import UpsertCommandForm from "@/components/forms/commands/UpsertCommandForm.vue";
  import { GROUP_FORM_ID } from "@/app.constants.ts";
  import { Button } from "@/components/ui/button";

  const props = defineProps<{
    viewTrigger?: boolean;
  }>();

  const createCategoryFormRef = ref<InstanceType<
    typeof UpsertCategoryForm
  > | null>(null);
</script>

<template>
  <FormDialog title="Create Category">
    <template v-if="viewTrigger" #trigger>
      <AddIcon />
      New Category
    </template>

    <template #default="{closeDialog}">
      <UpsertCommandForm @success="closeDialog" ref="createCategoryFormRef" />
    </template>

    <template #reset>
      <Button
        variant="outline"
        @click="createCategoryFormRef?.resetForm()"
        :is-pending="createCategoryFormRef?.isPending"
        :disabled="!createCategoryFormRef?.isDirty"
      >
        Reset
      </Button>
    </template>

    <template #submit>
      <Button
        type="submit"
        :form="GROUP_FORM_ID"
        :is-pending="createCategoryFormRef?.isPending"
        :disabled="!createCategoryFormRef?.isValid"
      >
        Create
      </Button>
    </template>
  </FormDialog>
</template>

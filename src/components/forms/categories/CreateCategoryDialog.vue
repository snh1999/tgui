<script setup lang="ts">
  import UpsertCategoryForm from "@/components/forms/categories/UpsertCategoryForm.vue";
  import { ref } from "vue";
  import FormDialog from "@/components/forms/common/FormDialog.vue";
  import { AddIcon } from "@/assets/Icons";
  import { CATEGORY_FORM_ID } from "@/app.constants.ts";
  import { Button } from "@/components/ui/button";
  import { ICategory } from "@/lib/api/api.types.ts";

  const props = defineProps<{
    viewTrigger?: boolean;
    category?: ICategory;
  }>();

  const createCategoryFormRef = ref<InstanceType<
    typeof UpsertCategoryForm
  > | null>(null);
</script>

<template>
  <FormDialog title="Create Category">
    <template v-if="$slots.default" #trigger>
      <slot />
    </template>
    <template v-if="viewTrigger" #trigger>
      <AddIcon />
      New Category
    </template>

    <template #default="{closeDialog}">
      <UpsertCategoryForm
        @success="closeDialog"
        ref="createCategoryFormRef"
        :category="category"
        isCreate
      />
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
        :form="CATEGORY_FORM_ID"
        :is-pending="createCategoryFormRef?.isPending"
        :disabled="!createCategoryFormRef?.isValid"
      >
        Create
      </Button>
    </template>
  </FormDialog>
</template>

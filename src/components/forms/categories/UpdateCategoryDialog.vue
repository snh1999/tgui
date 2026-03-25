<script setup lang="ts">
  import { ref } from "vue";
  import { CATEGORY_FORM_ID } from "@/app.constants.ts";
  import { EditIcon } from "@/assets/Icons";
  import UpsertCategoryForm from "@/components/forms/categories/UpsertCategoryForm.vue";
  import FormDialog from "@/components/forms/common/FormDialog.vue";
  import { Button } from "@/components/ui/button";
  import ErrorDisplay from "@/components/ui/tgui/ErrorDisplay.vue";
  import Loading from "@/components/ui/tgui/Loading.vue";
  import { useGetCategory } from "@/lib/api/composables/categories.ts";

  const props = defineProps<{
    id: number;
    viewTrigger?: boolean;
  }>();

  const {
    data: category,
    isPending,
    isError,
    error,
    refetch,
  } = useGetCategory(props.id);

  const updateCategoryRef = ref<InstanceType<typeof UpsertCategoryForm> | null>(
    null
  );
</script>

<template>
  <FormDialog title="Update category">
    <template v-if="viewTrigger" #trigger>
      <EditIcon />
      Edit Category
    </template>

    <template #default="{closeDialog}">
      <Loading v-if="isPending" />
      <ErrorDisplay v-if="isError" :error="error" :retry="refetch" />
      <UpsertCategoryForm
        v-if="category"
        :key="category.id"
        :category="category"
        @success="closeDialog"
        ref="updateCategoryRef"
      />
    </template>

    <template #reset>
      <Button
        variant="destructive"
        @click="updateCategoryRef?.resetForm()"
        :isPending="updateCategoryRef?.isPending"
      >
        Reset
      </Button>
    </template>

    <template #submit>
      <Button
        type="submit"
        :form="CATEGORY_FORM_ID"
        :isPending="updateCategoryRef?.isPending"
      >
        Update
      </Button>
    </template>
  </FormDialog>
</template>

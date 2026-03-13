<script setup lang="ts">
  import {
    IUpsertCategoryForm,
    useCategoryForm,
  } from "@/components/forms/categories/categories.helpers.ts";
  import Loading from "@/components/ui/tgui/Loading.vue";
  import { CATEGORY_FORM_ID } from "@/app.constants.ts";
  import { FieldGroup } from "@/components/ui/field";
  import { Input } from "@/components/ui/input";
  import FormField from "@/components/ui/tgui/inputs/FormField.vue";
  import IconPicker from "@/components/ui/tgui/inputs/IconPicker.vue";
  import ColorPickerInput from "@/components/ui/tgui/inputs/color-picker/ColorPickerInput.vue";

  const props = defineProps<IUpsertCategoryForm>();
  const emit = defineEmits<{ success: [] }>();

  const { resetForm, isPending, onSubmit, isDirty, isValid } = useCategoryForm(
    props,
    () => emit("success")
  );

  defineExpose({ resetForm, isPending, isDirty, isValid });
</script>

<template>
  <div>
    <Loading v-if="isPending" />
    <form :id="CATEGORY_FORM_ID" @submit="onSubmit">
      <FieldGroup>
        <FormField
          name="name"
          :form-id="CATEGORY_FORM_ID"
          label="Name"
          :class="[isPending ? 'pointer-events-none' : '']"
        >
          <Input placeholder="Category Name" />
        </FormField>

        <FormField name="icon" :form-id="CATEGORY_FORM_ID" label="Icon">
          <template #default="{ bindings }">
            <IconPicker v-bind="bindings" placeholder="Category Icon" />
          </template>
        </FormField>

        <FormField name="color" :form-id="CATEGORY_FORM_ID" label="Color">
          <template #default="{ bindings }">
            <ColorPickerInput v-bind="bindings" />
          </template>
        </FormField>
      </FieldGroup>
    </form>
  </div>
</template>

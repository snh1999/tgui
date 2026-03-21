<script setup lang="ts">
  import { COMMAND_FORM_ID, GROUP_FORM_ID } from "@/app.constants.ts";
  import {
    IUpsertGroupForm,
    useGroupForm,
  } from "@/components/forms/groups/groups.helpers.ts";
  import { FieldGroup } from "@/components/ui/field";
  import { Input } from "@/components/ui/input";
  import FormField from "@/components/ui/tgui/inputs/FormField.vue";
  import Loading from "@/components/ui/tgui/Loading.vue";
  import MapInput from "@/components/ui/tgui/inputs/MapInput.vue";
  import {
    InputGroup,
    InputGroupAddon,
    InputGroupText,
    InputGroupTextarea,
  } from "@/components/ui/input-group";
  import DirectoryPicker from "@/components/ui/tgui/inputs/DirectoryPicker.vue";
  import IconPicker from "@/components/ui/tgui/inputs/IconPicker.vue";
  import ShellSelect from "@/components/forms/common/ShellSelect.vue";
  import GroupSelect from "@/components/forms/common/GroupSelect.vue";
  import CategorySelect from "@/components/forms/common/CategorySelect.vue";

  const props = defineProps<IUpsertGroupForm>();
  const emit = defineEmits<{ success: [] }>();

  const { resetForm, isPending, onSubmit, isDirty, isValid } = useGroupForm(
    props,
    () => emit("success")
  );

  defineExpose({ resetForm, isPending, isDirty, isValid });
</script>

<template>
  <div>
    <Loading v-if="isPending" />

    <form :id="GROUP_FORM_ID" @submit="onSubmit">
      <FieldGroup>
        <FormField
          name="name"
          :form-id="GROUP_FORM_ID"
          label="Name"
          :class="[isPending ? 'pointer-events-none' : '']"
        >
          <Input placeholder="Group Name" />
        </FormField>

        <FormField
          name="workingDirectory"
          :form-id="GROUP_FORM_ID"
          label="Working Directory"
        >
          <template #default="{ bindings }">
            <DirectoryPicker
              v-bind="bindings"
              placeholder="Select the location where you want to execute the command"
            />
          </template>
        </FormField>

        <FormField
          name="parentGroupId"
          :form-id="COMMAND_FORM_ID"
          label="Group"
        >
          <template #default="{ bindings }">
            <GroupSelect v-bind="bindings" placeholder="Select a group" />
          </template>
        </FormField>

        <FormField
          name="categoryId"
          :form-id="COMMAND_FORM_ID"
          label="Category"
        >
          <template #default="{ bindings }">
            <CategorySelect v-bind="bindings" placeholder="Select a Category" />
          </template>
        </FormField>

        <FormField name="icon" :form-id="GROUP_FORM_ID" label="Icon">
          <template #default="{ bindings }">
            <IconPicker
              v-bind="bindings"
              placeholder="Select an Icon for the Group"
            />
          </template>
        </FormField>

        <FormField name="shell" :form-id="COMMAND_FORM_ID" label="Shell">
          <template #default="{ bindings }">
            <ShellSelect
              v-bind="bindings"
              placeholder="Select preferred shell"
            />
          </template>
        </FormField>

        <MapInput
          fieldName="envVars"
          label="Environment Variables"
          keyPlaceholder="Enter Key"
          valuePlaceholder="Enter value"
        />

        <FormField
          name="description"
          :form-id="GROUP_FORM_ID"
          label="Description"
        >
          <template #default="{ bindings, field }">
            <InputGroup>
              <InputGroupTextarea
                v-bind="bindings"
                placeholder="I'm having an issue..."
                :rows="6"
                class="min-h-24 resize-none"
              />
              <InputGroupAddon align="block-end">
                <InputGroupText class="tabular-nums">
                  {{ field.value?.length || 0 }} characters
                </InputGroupText>
              </InputGroupAddon>
            </InputGroup>
          </template>
        </FormField>
      </FieldGroup>
    </form>
  </div>
</template>

<script setup lang="ts">
  import { COMMAND_FORM_ID } from "@/app.constants.ts";
  import {
    IUpsertCommandForm,
    useCommandForm,
  } from "@/components/forms/commands/commands.helpers.ts";
  import CategorySelect from "@/components/forms/common/CategorySelect.vue";
  import GroupSelect from "@/components/forms/common/GroupSelect.vue";
  import ShellSelect from "@/components/forms/common/ShellSelect.vue";
  import { FieldGroup } from "@/components/ui/field";
  import { Input } from "@/components/ui/input";
  import {
    InputGroup,
    InputGroupAddon,
    InputGroupText,
    InputGroupTextarea,
  } from "@/components/ui/input-group";
  import ArrayInput from "@/components/ui/tgui/inputs/ArrayInput.vue";
  import DirectoryPicker from "@/components/ui/tgui/inputs/DirectoryPicker.vue";
  import FormField from "@/components/ui/tgui/inputs/FormField.vue";
  import MapInput from "@/components/ui/tgui/inputs/MapInput.vue";
  import Loading from "@/components/ui/tgui/Loading.vue";

  const props = defineProps<IUpsertCommandForm>();
  const emit = defineEmits<{ success: [] }>();
  const { resetForm, isPending, onSubmit, isDirty, isValid } = useCommandForm(
    props,
    () => emit("success")
  );
  defineExpose({ resetForm, isPending, isValid, isDirty });
</script>

<template>
  <div>
    <Loading v-if="isPending" />

    <form :id="COMMAND_FORM_ID" @submit="onSubmit">
      <FieldGroup>
        <FormField name="name" :form-id="COMMAND_FORM_ID" label="Name">
          <Input placeholder="Command Name" />
        </FormField>

        <FormField
          name="command"
          :form-id="COMMAND_FORM_ID"
          label="Command Text"
        >
          <Input placeholder="Command to execute" autofocus />
        </FormField>

        <ArrayInput
          fieldName="arguments"
          label="Arguments"
          placeholder="Add Argument"
          addButtonText="Add Argument"
        />

        <FormField
          name="workingDirectory"
          :form-id="COMMAND_FORM_ID"
          label="Working Directory"
        >
          <template #default="{ bindings }">
            <DirectoryPicker
              v-bind="bindings"
              placeholder="Select the location where you want to execute the command"
            />
          </template>
        </FormField>

        <FormField name="groupId" :form-id="COMMAND_FORM_ID" label="Group">
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
          addButtonText="Add Variable"
        />

        <FormField
          name="description"
          :form-id="COMMAND_FORM_ID"
          label="Description"
        >
          <template #default="{ bindings, field }">
            <InputGroup>
              <InputGroupTextarea
                v-bind="bindings"
                placeholder="I'm having an issue..."
                :rows="3"
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

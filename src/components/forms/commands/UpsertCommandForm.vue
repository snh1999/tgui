<script setup lang="ts">
  import { COMMAND_FORM_ID } from "@/app.constants.ts";
  import {
    IUpsertCommandForm,
    useCommandForm,
    useCommandLineSync,
  } from "@/components/forms/commands/commands.helpers.ts";
  import CategorySelect from "@/components/forms/common/CategorySelect.vue";
  import GroupSelect from "@/components/forms/common/GroupSelect.vue";
  import ShellSelect from "@/components/forms/common/ShellSelect.vue";
  import {
    Field,
    FieldDescription,
    FieldError,
    FieldGroup,
    FieldLabel,
  } from "@/components/ui/field";
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
  import { Separator } from "@/components/ui/separator";
  import { useExplainCommand } from "@/lib/api/composables/commands.ts";
  import { computed, ref } from "vue";

  const props = defineProps<IUpsertCommandForm>();
  const emit = defineEmits<{ success: [] }>();

  const inputForExplain = ref(props.commandText ?? "");
  const { data: commandExplanation } = useExplainCommand(inputForExplain);

  const commandName = computed(() => {
    const summary = commandExplanation.value?.summary;
    if (summary && summary !== "Unrecognized command") {
      return summary;
    }
    return "";
  });

  const {
    resetForm,
    isPending,
    handleFormSubmit,
    isDirty,
    isValid,
    values,
    setFieldValue,
  } = useCommandForm(props, () => emit("success"), commandName);

  const { combined, onCombinedChange, error } = useCommandLineSync(
    values,
    setFieldValue,
    props.commandText
  );

  defineExpose({ resetForm, isPending, isValid, isDirty });
</script>

<template>
  <div>
    <Loading v-if="isPending" />

    <form :id="COMMAND_FORM_ID" @submit="handleFormSubmit">
      <FieldGroup>
        <FormField name="name" :form-id="COMMAND_FORM_ID" label="Name">
          <Input placeholder="Command Name" />
        </FormField>

        <FieldGroup>
          <Field :data-invalid="error">
            <FieldLabel>Command Text</FieldLabel>
            <FieldDescription>
              {{ commandName || "Type the full command here, or use the fields below." }}
            </FieldDescription>
            <Input
              v-model="combined"
              placeholder='e.g. echo "hello world" or git commit -m "msg"'
              @input="onCombinedChange"
              @blur="inputForExplain = combined"
            />
            <FieldError v-if="error">{{ error }}</FieldError>
          </Field>

          <div class="grid grid-cols-2 gap-4">
            <FormField
              name="command"
              :form-id="COMMAND_FORM_ID"
              label="Executable"
            >
              <Input placeholder="Executable, e.g. echo" />
            </FormField>

            <ArrayInput
              fieldName="arguments"
              label="Arguments"
              placeholder="Add Argument"
              addButtonText="Add Argument"
            />
          </div>
        </FieldGroup>

        <Separator />
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

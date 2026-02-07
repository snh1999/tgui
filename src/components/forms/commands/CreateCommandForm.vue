<script setup lang="ts">
import {toTypedSchema} from "@vee-validate/zod";
import {useForm} from "vee-validate";
import {effect, toRaw} from "vue";
import {commandFormSchema} from "@/components/forms/commands/commands.helpers.ts";
import {FieldGroup} from "@/components/ui/field";
import {Input} from "@/components/ui/input";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupText,
  InputGroupTextarea,
} from "@/components/ui/input-group";
import {COMMAND_FORM_ID} from "@/app.constants.ts";
import FormField from "@/components/ui/tgui/FormField.vue";
import ArrayInput from "@/components/ui/tgui/ArrayInput.vue";
import MapInput from "@/components/ui/tgui/MapInput.vue";
import {transformEnvVars} from "@/lib/helpers.ts";
import {invoke} from "@tauri-apps/api/core";

const props = defineProps<{
  onSuccess?: () => void;
}>();

const {handleSubmit, resetForm, errors} = useForm({
  validationSchema: toTypedSchema(commandFormSchema),
  initialValues: {
    name: "",
    command: "",
    id: 0,
    position: 0,
    env_vars: [],
    arguments: [""],
  },
});

effect(() => {
  console.log(toRaw(errors.value));
});

const onSubmit = handleSubmit(async (rawData) => {
  const data = transformEnvVars(rawData);

  console.log(data);
  const response = await invoke('create_command', {cmd: data})
  console.log(response);

  // if (props.onSuccess) {
  //   props.onSuccess();
  // }
});

defineExpose({resetForm});
</script>

<template>
  <form :id="COMMAND_FORM_ID" @submit="onSubmit">
    <FieldGroup>
      <FormField name="name" :form-id="COMMAND_FORM_ID" label="Name">
        <Input placeholder="Command Name"/>
      </FormField>

      <FormField name="command" :form-id="COMMAND_FORM_ID" label="Command Text">
        <Input placeholder="Command to execute"/>
      </FormField>

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

      <ArrayInput
          name="arguments"
          label="Arguments"
          placeholder="Add Argument"
      />
      <MapInput
          name="env_vars"
          label="Environment Variables"
          keyPlaceholder="Enter Key"
          valuePlaceholder="Enter value"
      />

      <!--      TODO add groupid, category id-->

      <!--      TODO: add file picker-->
      <FormField
          name="working_directory"
          :form-id="COMMAND_FORM_ID"
          label="Working Directory"
      >
        <Input
            placeholder="Select the location where you want to execute the command"
        />
      </FormField>

      <FormField name="shell" :form-id="COMMAND_FORM_ID" label="Shell">
        <Input placeholder="Choose default shell"/>
      </FormField>
    </FieldGroup>
  </form>
</template>

<script setup lang="ts">
import {toTypedSchema} from "@vee-validate/zod";
import {useForm} from "vee-validate";
import {COMMAND_FORM_ID} from "@/app.constants.ts";
import {commandFormSchema} from "@/components/forms/commands/commands.helpers.ts";
import {FieldGroup} from "@/components/ui/field";
import {Input} from "@/components/ui/input";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupText,
  InputGroupTextarea,
} from "@/components/ui/input-group";
import ArrayInput from "@/components/ui/tgui/ArrayInput.vue";
import FormField from "@/components/ui/tgui/FormField.vue";
import MapInput from "@/components/ui/tgui/MapInput.vue";
import {useCreateCommand, useUpdateCommand} from "@/lib/api/composables/commands.ts";
import {transformEnvVars} from "@/lib/helpers.ts";
import Loading from "@/components/ui/tgui/Loading.vue";
import {ICommand} from "@/lib/api/api.types.ts";
import {computed} from "vue";

const {command, onSuccess} = defineProps<{
  onSuccess?: () => void;
  command?: ICommand;
}>();

const {handleSubmit, resetForm} = useForm({
  validationSchema: toTypedSchema(commandFormSchema),
  initialValues: command ?? {
    name: "",
    command: "",
    id: 0,
    working_directory: "~",
    position: 0,
    env_vars: [],
    arguments: [""],
  },
});

const {mutate: createCommand, isPending: isCreatePending} = useCreateCommand();
const {mutate: updateCommand, isPending: isUpdatePending} = useUpdateCommand();

const isPending = computed(() => command ? isUpdatePending.value : isCreatePending.value);

const onSubmit = handleSubmit((rawData) => {
  const data = transformEnvVars(rawData);
  const mutationOptions = {
    onSuccess: () => {
      if (!command) {
        resetForm();
      }
      onSuccess?.();
    }
  };

  if (command) {
    updateCommand({id: command.id, payload: data}, mutationOptions);
  } else {
    createCommand(data, mutationOptions);
  }
});

defineExpose({resetForm, isPending});
</script>

<template>
  <div>
    <Loading :isVisible="isPending"/>

    <form :id="COMMAND_FORM_ID" @submit="onSubmit">
      <FieldGroup>
        <FormField
            name="name"
            :form-id="COMMAND_FORM_ID"
            label="Name"
            :class="[isPending ? 'pointer-events-none' : '']"
        >
          <Input placeholder="Command Name"/>
        </FormField>

        <FormField
            name="command"
            :form-id="COMMAND_FORM_ID"
            label="Command Text"
        >
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
  </div>
</template>

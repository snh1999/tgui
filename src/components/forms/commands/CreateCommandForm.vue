<script setup lang="ts">
import {toTypedSchema} from "@vee-validate/zod";
import {useForm} from "vee-validate";
import {effect, h, toRaw} from "vue";
import {toast} from "vue-sonner";
import {commandFormSchema} from "@/components/forms/commands/commands.helpers.ts";
import {Button} from "@/components/ui/button";
import {Field, FieldGroup,} from "@/components/ui/field";
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

const {handleSubmit, resetForm, errors} = useForm({
  validationSchema: toTypedSchema(commandFormSchema),
  initialValues: {
    name: "",
    command: "",
    id: 0,
    position: 0,
    env_vars: [],
    arguments: [""]
  },
});

effect(() => {
  console.log(toRaw(errors.value))
})

const onSubmit = handleSubmit((rawData) => {
  const data = transformEnvVars(rawData);
  console.log(data);
  toast("You submitted the following values:", {
    description: h(
        "pre",
        {
          class:
              "bg-code text-code-foreground mt-2 w-[320px] overflow-x-auto rounded-md p-4",
        },
        h("code", JSON.stringify(data, null, 2))
    ),
    position: "bottom-right",
    class: "flex flex-col gap-2",
    style: {
      "--border-radius": "calc(var(--radius)  + 4px)",
    },
  });
});
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
      <FormField name="working_directory" :form-id="COMMAND_FORM_ID" label="Working Directory">
        <Input placeholder="Select the location where you want to execute the command"/>
      </FormField>

      <FormField name="shell" :form-id="COMMAND_FORM_ID" label="Shell">
        <Input placeholder="Choose default shell"/>
      </FormField>

    </FieldGroup>
  </form>
  <Field orientation="horizontal">
    <Button type="button" variant="outline" @click="resetForm">Reset</Button>
    <Button type="submit" :form="COMMAND_FORM_ID">Submit</Button>
  </Field>
</template>

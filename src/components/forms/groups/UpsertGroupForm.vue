<script setup lang="ts">
  import { GROUP_FORM_ID } from "@/app.constants.ts";
  import {
    IUpsertGroupForm,
    useGroupForm,
  } from "@/components/forms/groups/groups.helpers.ts";
  import { FieldGroup } from "@/components/ui/field";
  import { Input } from "@/components/ui/input";
  import {
    InputGroup,
    InputGroupAddon,
    InputGroupText,
    InputGroupTextarea,
  } from "@/components/ui/input-group";
  import FormField from "@/components/ui/tgui/FormField.vue";
  import Loading from "@/components/ui/tgui/Loading.vue";
  import MapInput from "@/components/ui/tgui/MapInput.vue";

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
        <MapInput
          fieldName="envVars"
          label="Environment Variables"
          keyPlaceholder="Enter Key"
          valuePlaceholder="Enter value"
        />

        <!--      TODO add parent groupid, category id-->

        <!--      TODO: add file picker-->
        <FormField
          name="workingDirectory"
          :form-id="GROUP_FORM_ID"
          label="Working Directory"
        >
          <Input
            placeholder="Select the location where you want to execute the commands in the group"
          />
        </FormField>

        <FormField name="shell" :form-id="GROUP_FORM_ID" label="Shell">
          <Input placeholder="Choose default shell" />
        </FormField>
      </FieldGroup>
    </form>
  </div>
</template>

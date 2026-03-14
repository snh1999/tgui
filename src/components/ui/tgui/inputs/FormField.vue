<script setup lang="ts">
  import { Field as VeeField } from "vee-validate";
  import {
    Field,
    FieldDescription,
    FieldError,
    FieldLabel,
  } from "@/components/ui/field";
  import { Input } from "@/components/ui/input";
  import { cloneVNode, computed, h, isVNode, useSlots } from "vue";

  /**
   * FormField
   *
   * This is shadcn form field wrapper that auto-binds vee-validate state to any input component.
   * reducing boilerplate for labels, error display, and accessibility attributes.
   *
   * @example Simple usage - auto-binds to single input (first child)
   * <FormField name="email" label="Email">
   *   <Input type="email" placeholder="user@example.com" />
   * </FormField>
   *
   * @example Complex layout - manual binding via scoped slot
   * <FormField name="bio" label="Bio">
   *   <template #default="{ bindings, field }">
   *     <InputGroup>
   *       <Textarea v-bind="bindings" />
   *       <InputGroupAddon>{{ field.value?.length }}/500</InputGroupAddon>
   *     </InputGroup>
   *   </template>
   * </FormField>
   *
   * @example Checkbox, Switch, Radio - all work
   * <FormField name="terms" label="Terms">
   *   <Checkbox label="I agree" />
   * </FormField>
   *
   * @example With tgui component
   * <FormField name="role" label="Role" :as="Select">
   *   <SelectItem value="admin">Admin</SelectItem>
   *   <SelectItem value="user">User</SelectItem>
   * </FormField>
   */

  const props = defineProps<{
    name: string;
    label?: string;
    formId?: string;
    description?: string;
  }>();

  const slots = useSlots();
  const fieldId = computed(() =>
    props.formId ? `${props.formId}-${props.name}` : `field-${props.name}`
  );

  // NOTE: consider using a type prop instead of manipulating the slot node
  // can not pass/append values directly via slot concisely, so that is the option
  const AutoInput = ({ field, errors }: { field: any; errors: string[] }) => {
    const bindings = {
      id: fieldId.value,
      name: props.name,
      ...field,
      modelValue: field.value,
      "onUpdate:modelValue": field.onChange,
      "aria-invalid": !!errors.length,
    };

    if (!slots.default) {
      return h(Input, bindings);
    }

    const slotNodes = slots.default({
      field,
      errors,
      bindings,
      id: fieldId.value,
    });

    if (!slotNodes?.length) {
      return null;
    }

    const firstNode = slotNodes[0];

    if (isVNode(firstNode)) {
      return cloneVNode(slotNodes[0], bindings);
    }

    return null;
  };
</script>

<template>
  <VeeField v-slot="{ field, errors }" :name="name">
    <Field :data-invalid="!!errors.length">
      <FieldLabel v-if="label" :for="fieldId">{{ label }}</FieldLabel>

      <AutoInput :field="field" :errors="errors" />

      <FieldDescription v-if="description || $slots.description">
        <slot name="description">{{ description }}</slot>
      </FieldDescription>

      <FieldError v-if="errors.length" :errors="errors" />
    </Field>
  </VeeField>
</template>

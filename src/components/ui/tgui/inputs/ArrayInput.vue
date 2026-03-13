<script setup lang="ts">
import {Field as VeeField, FieldArray as VeeFieldArray} from "vee-validate";
import {AddIcon, DeleteIcon} from "@/assets/Icons.ts"; // adjust path
import {Button} from "@/components/ui/button";
import {Field, FieldError, FieldLabel} from "@/components/ui/field";
import {Input} from "@/components/ui/input";

defineProps<{
    fieldName: string;
    label?: string;
    addButtonText?: string;
  }>();
</script>

<template>
  <VeeFieldArray v-slot="{ fields, push, remove }" :name="fieldName">
    <div class="space-y-2">
      <div class="flex justify-between items-center">
        <FieldLabel v-if="label">{{ label }}</FieldLabel>
      </div>

      <div class="space-y-2">
        <VeeField
          v-for="(field, index) in fields"
          :key="field.key"
          v-slot="{ field: inputField, errors }"
          :name="`${fieldName}[${index}]`"
        >
          <Field orientation="horizontal" :data-invalid="!!errors.length">
            <Input
              v-bind="inputField"
              :model-value="inputField.value"
              placeholder="Argument or &quot;quoted argument&quot;"
              :aria-invalid="!!errors.length"
              class="flex-1"
            />
            <Button
                type="button"
                class="text-destructive hover:opacity-75 shrink-0 mt-0"
                variant="link"
                size="icon-sm"
              @click="remove(index)"
            >
              <DeleteIcon />
            </Button>
            <FieldError v-if="errors.length" :errors="errors" />
          </Field>
        </VeeField>
      </div>

      <Button
          type="button"
          @click="push('')"
      >
        <AddIcon />  {{addButtonText??  `Add ${label}`}}
      </Button>
    </div>
  </VeeFieldArray>
</template>

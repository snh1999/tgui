<script setup lang="ts">
  import { Field as VeeField, FieldArray as VeeFieldArray } from "vee-validate";
  import { AddIcon, DeleteIcon } from "@/assets/Icons.ts";
  import { Button } from "@/components/ui/button";
  import { Field, FieldError, FieldLabel } from "@/components/ui/field";
  import { Input } from "@/components/ui/input";

  defineProps<{
    fieldName: string;
    label?: string;
    keyPlaceholder?: string;
    valuePlaceholder?: string;
  }>();
</script>

<template>
  <VeeFieldArray v-slot="{ fields, push, remove }" :name="fieldName">
    <div class="space-y-2">
      <div class="flex justify-between items-center">
        <FieldLabel v-if="label">{{ label }}</FieldLabel>
        <Button
          type="button"
          variant="outline"
          size="icon-sm"
          @click="push({ key: '', value: '' })"
        >
          <AddIcon class="text-primary" />
        </Button>
      </div>

      <div class="space-y-2">
        <div
          v-for="(field, index) in fields"
          :key="field.key"
          class="flex gap-2 items-start"
        >
          <!-- Key Field -->
          <VeeField
            v-slot="{ field: keyField, errors: keyErrors }"
            :name="`${fieldName}[${index}].key`"
          >
            <Field class="flex-1" :data-invalid="!!keyErrors.length">
              <Input
                v-bind="keyField"
                :model-value="keyField.value"
                :placeholder="keyPlaceholder || 'KEY'"
                :aria-invalid="!!keyErrors.length"
              />
              <FieldError v-if="keyErrors.length" :errors="keyErrors" />
            </Field>
          </VeeField>

          <!-- Value Field -->
          <VeeField
            v-slot="{ field: valueField, errors: valueErrors }"
            :name="`${fieldName}[${index}].value`"
          >
            <Field class="flex-1" :data-invalid="!!valueErrors.length">
              <Input
                v-bind="valueField"
                :model-value="valueField.value"
                :placeholder="valuePlaceholder || 'value'"
                :aria-invalid="!!valueErrors.length"
              />
              <FieldError v-if="valueErrors.length" :errors="valueErrors" />
            </Field>
          </VeeField>

          <!-- Remove Button -->
          <Button
            type="button"
            class="text-destructive shrink-0 mt-0"
            variant="outline"
            size="icon-sm"
            @click="remove(index)"
          >
            <DeleteIcon />
          </Button>
        </div>
      </div>
    </div>
  </VeeFieldArray>
</template>

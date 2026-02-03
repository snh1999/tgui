<script setup lang="ts">
import {Field as VeeField, FieldArray as VeeFieldArray} from 'vee-validate';
import {Input} from '@/components/ui/input';
import {Field, FieldError, FieldLabel} from '@/components/ui/field';
import {Button} from '@/components/ui/button';
import {AddIcon, DeleteIcon} from '@/assets/Icons.ts'; // adjust path

defineProps<{
  name: string;
  label?: string;
}>();
</script>

<template>
  <VeeFieldArray v-slot="{ fields, push, remove }" :name="name">
    <div class="space-y-2">
      <div class="flex justify-between items-center">
        <FieldLabel v-if="label">{{ label }}</FieldLabel>
        <Button
            type="button"
            variant="outline"
            size="icon-sm"
            @click="push('')"
        >
          <AddIcon class="text-primary"/>
        </Button>
      </div>

      <div class="space-y-2">
        <VeeField
            v-for="(field, index) in fields"
            :key="field.key"
            v-slot="{ field: inputField, errors }"
            :name="`${name}[${index}]`"
        >
          <Field orientation="horizontal" :data-invalid="!!errors.length">
            <Input
                v-bind="inputField"
                placeholder="Argument or &quot;quoted argument&quot;"
                :aria-invalid="!!errors.length"
                class="flex-1"
            />
            <Button
                type="button"
                class="text-destructive shrink-0"
                variant="outline"
                size="icon-sm"
                @click="remove(index)"
            >
              <DeleteIcon/>
            </Button>
            <FieldError v-if="errors.length" :errors="errors"/>
          </Field>
        </VeeField>
      </div>
    </div>
  </VeeFieldArray>
</template>

<script setup lang="ts">
  import { computed } from "vue";
  import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectLabel,
    SelectTrigger,
    SelectValue,
  } from "@/components/ui/select";
  import {
    IInputEmits,
    IInputProps,
  } from "@/components/ui/tgui/inputs/tgui-input.types.ts";

  interface IProps {
    id: number | string;
    value: number | string;
    name: string;
  }

  const props = defineProps<
    IInputProps & {
      data?: IProps[];
    }
  >();
  const emit = defineEmits<IInputEmits>();

  const model = computed({
    get: () => props.modelValue || "",
    set: (v) => emit("update:modelValue", v ?? ""),
  });
</script>

<template>
  <Select v-model="model">
    <SelectTrigger class="w-45">
      <SelectValue :placeholder="placeholder ?? `Select one of the ${name}`" />
    </SelectTrigger>
    <SelectContent>
      <SelectGroup>
        <SelectLabel>{{ name }}</SelectLabel>
        <SelectItem
          v-for="entry in (data??[])"
          :key="entry.id"
          :value="entry.value"
        >
          {{ entry.name }}
        </SelectItem>
      </SelectGroup>
    </SelectContent>
  </Select>
</template>

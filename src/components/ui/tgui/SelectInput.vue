<script setup lang="ts">
  import { computed } from "vue";
  import GroupCategoryLine from "@/components/shared/GroupCategoryLine.vue";
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
  import { ICategory } from "@/lib/api/api.types.ts";

  interface IProps extends Omit<ICategory, "id" | "createdAt"> {
    id: number | string;
    value: number | string;
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
    <SelectTrigger class="w-40">
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
          <GroupCategoryLine
            v-if="entry.icon || entry.color"
            :element="entry"
          />
          <span v-else>{{ entry.name }}</span>
        </SelectItem>
      </SelectGroup>
    </SelectContent>
  </Select>
</template>

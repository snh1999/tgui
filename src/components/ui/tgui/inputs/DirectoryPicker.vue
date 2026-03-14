<script setup lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { X as CloseIcon } from "lucide-vue-next";
  import { computed } from "vue";
  import { DirectoryIcon } from "@/assets/Icons.ts";
  import {
    InputGroup,
    InputGroupAddon,
    InputGroupButton,
    InputGroupInput,
  } from "@/components/ui/input-group";
  import {
    IInputEmits,
    IInputProps,
  } from "@/components/ui/tgui/inputs/tgui-input.types.ts";

  const props = defineProps<IInputProps>();
  const emit = defineEmits<IInputEmits>();

  const model = computed({
    get: () => props.modelValue || "",
    set: (v) => emit("update:modelValue", v),
  });

  async function pickDir() {
    if (props.disabled) {
      return;
    }
    const selected = await open({ directory: true });

    if (selected) {
      emit("update:modelValue", selected);
    }
  }

  function clearDir() {
    emit("update:modelValue", "");
  }
</script>

<template>
  <InputGroup class="h-10">
    <InputGroupInput
      :id="id"
      :name="name"
      v-model="model"
      :placeholder="placeholder || 'Select a directory...'"
      readonly
      :disabled="disabled"
      class="flex-1 truncate"
      :class="{ 'text-muted-foreground': !modelValue }"
    />
    <InputGroupAddon align="inline-end">
      <InputGroupButton
        v-if="modelValue && !disabled"
        type="button"
        variant="ghost"
        @click.stop="clearDir"
      >
        <CloseIcon />
      </InputGroupButton>
      <InputGroupButton
        type="button"
        size="sm"
        :disabled="disabled"
        class="shrink-0"
        @click="pickDir"
        variant="ghost"
      >
        <DirectoryIcon />
        {{ modelValue ? 'Change' : 'Browse' }}
      </InputGroupButton>
    </InputGroupAddon>
  </InputGroup>
</template>

<script lang="ts" setup>
  import { Brush, X as ClearIcon, Paintbrush } from "lucide-vue-next";
  import { computed, ref } from "vue";
  import {
    InputGroup,
    InputGroupAddon,
    InputGroupButton,
    InputGroupInput,
  } from "@/components/ui/input-group";
  import {
    Popover,
    PopoverContent,
    PopoverTrigger,
  } from "@/components/ui/popover";
  import ColorPicker from "@/components/ui/tgui/inputs/color-picker/ColorPicker.vue";
  import {
    IInputEmits,
    IInputProps,
  } from "@/components/ui/tgui/inputs/tgui-input.types.ts";

  const props = defineProps<IInputProps>();
  const emit = defineEmits<IInputEmits>();

  const open = ref(false);

  const model = computed({
    get: () => props.modelValue || "",
    set: (v) => emit("update:modelValue", v),
  });

  function clear() {
    emit("update:modelValue", "");
  }
</script>

<template>
  <Popover v-model:open="open">
    <InputGroup class="h-10">
      <InputGroupInput
        :id="id"
        :class="{ 'text-muted-foreground': !modelValue }"
        :disabled="disabled"
        :name="name"
        :placeholder="placeholder || 'Select a Color...'"
        v-model="model"
        class="flex-1 truncate"
        readonly
      />
      <InputGroupAddon>
        <div
          v-if="modelValue"
          :style="{backgroundColor: modelValue as string}"
          class="h-5 w-5 rounded-md"
        />
        <Paintbrush v-else />
      </InputGroupAddon>

      <InputGroupAddon align="inline-end">
        <InputGroupButton
          v-if="modelValue && !disabled"
          type="button"
          variant="ghost"
          @click.stop="clear"
        >
          <ClearIcon />
        </InputGroupButton>

        <PopoverTrigger :disabled="disabled" as-child>
          <InputGroupButton
            :disabled="disabled"
            class="shrink-0"
            size="sm"
            type="button"
            variant="ghost"
          >
            <Brush />
            {{ modelValue ? 'Change' : 'Pick' }}
          </InputGroupButton>
        </PopoverTrigger>
      </InputGroupAddon>
    </InputGroup>
    <PopoverContent :side-offset="4" align="start" class="w-80 p-0">
      <ColorPicker
        :model-value="modelValue"
        @update:model-value="emit('update:modelValue', $event)"
      />
    </PopoverContent>
  </Popover>
</template>

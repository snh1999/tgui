<script lang="ts" setup>
  import { ref } from "vue";
  import {
    Popover,
    PopoverContent,
    PopoverTrigger,
  } from "@/components/ui/popover";
  import {
    InputGroup,
    InputGroupAddon,
    InputGroupButton,
    InputGroupInput,
  } from "@/components/ui/input-group";

  import { Brush, Paintbrush, X as ClearIcon } from "lucide-vue-next";
  import ColorPicker from "@/components/ui/tgui/inputs/color-picker/ColorPicker.vue";
  import {
    IInputEmits,
    IInputProps,
  } from "@/components/ui/tgui/inputs/tgui-input.types.ts";

  const props = defineProps<IInputProps>();
  const emit = defineEmits<IInputEmits>();

  const open = ref(false);

  function clear() {
    emit("update:modelValue", "");
  }
</script>

<template>
  <Popover v-model:open="open">
    <InputGroup class="h-10">
      <InputGroupInput
        readonly
        :id="id"
        :name="name"
        :value="modelValue || ''"
        :placeholder="placeholder || 'Select a Color...'"
        :disabled="disabled"
        class="flex-1 truncate"
        :class="{ 'text-muted-foreground': !modelValue }"
      />
      <InputGroupAddon>
        <div
          v-if="modelValue"
          class="h-5 w-5 rounded-md"
          :style="{backgroundColor: modelValue as string}"
        />
        <Paintbrush v-else />
      </InputGroupAddon>

      <InputGroupAddon align="inline-end">
        <InputGroupButton
          v-if="modelValue && !disabled"
          variant="ghost"
          @click.stop="clear"
        >
          <ClearIcon />
        </InputGroupButton>

        <PopoverTrigger as-child :disabled="disabled">
          <InputGroupButton
            type="button"
            size="sm"
            :disabled="disabled"
            class="shrink-0"
            variant="ghost"
          >
            <Brush />
            {{ modelValue ? 'Change' : 'Pick' }}
          </InputGroupButton>
        </PopoverTrigger>
      </InputGroupAddon>
    </InputGroup>
    <PopoverContent class="w-80 p-0" align="start" :side-offset="4">
      <ColorPicker
        :model-value="modelValue"
        @update:model-value="emit('update:modelValue', $event)"
      />
    </PopoverContent>
  </Popover>
</template>

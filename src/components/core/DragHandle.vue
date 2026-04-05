<script setup lang="ts">
  import { computed } from "vue";
  import { DragIcon } from "@/assets/Icons.ts";

  const props = defineProps<{ modelValue: number }>();
  const emit = defineEmits<{ "update:modelValue": [value: number] }>();

  const handleLeft = computed(() => props.modelValue - 2);

  const startDrag = (e: MouseEvent) => {
    e.preventDefault();
    const startX = e.clientX;
    const startWidth = props.modelValue;

    const onMove = (e: MouseEvent) => {
      const newWidth = Math.max(
        160,
        Math.min(480, startWidth + (e.clientX - startX))
      );
      emit("update:modelValue", newWidth);
    };

    const onUp = () => {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    };

    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  };
</script>

<template>
  <div
    class="fixed top-10 h-screen w-1 active:cursor-grabbing hover:bg-primary/40 cursor-col-resize z-100 transition-colors"
    :style="{ left: `${handleLeft}px` }"
    @mousedown="startDrag"
  >
    <DragIcon
      class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2"
    />
  </div>
</template>

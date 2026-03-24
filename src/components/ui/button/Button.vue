<script setup lang="ts">
  import type { PrimitiveProps } from "reka-ui";
  import { Primitive } from "reka-ui";
  import type { HTMLAttributes } from "vue";
  import { Spinner } from "@/components/ui/spinner";
  import { cn } from "@/lib/utils";
  import type { ButtonVariants } from ".";
  import { buttonVariants } from ".";

  interface Props extends PrimitiveProps {
    variant?: ButtonVariants["variant"];
    size?: ButtonVariants["size"];
    class?: HTMLAttributes["class"];
    isPending?: boolean;
  }

  const props = withDefaults(defineProps<Props>(), {
    as: "button",
  });
</script>

<template>
  <Primitive
    data-slot="button"
    :as="as"
    :as-child="asChild"
    :disabled="isPending"
    :class="cn(buttonVariants({ variant, size }), props.class)"
    class="cursor-pointer text-[13px] font-semibold whitespace-nowrap"
  >
    <Spinner v-if="isPending" />
    <slot />
  </Primitive>
</template>

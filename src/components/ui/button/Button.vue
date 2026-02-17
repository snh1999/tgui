<script setup lang="ts">
  import type { PrimitiveProps } from "reka-ui";
  import { Primitive } from "reka-ui";
  import type { HTMLAttributes } from "vue";
  import { cn } from "@/lib/utils";
  import type { ButtonVariants } from ".";
  import { buttonVariants } from ".";
  import { Spinner } from "@/components/ui/spinner";

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
  >
    <Spinner v-if="isPending" />
    <slot />
  </Primitive>
</template>

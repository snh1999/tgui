<script setup lang="ts">
  import { computed } from "vue";
  import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
  import { Button } from "@/components/ui/button";
  import { AlertCircle, RefreshCw, X } from "lucide-vue-next";

  interface Props {
    error?: Error | { message: string; code?: string } | string | null;
    title?: string;
    retry?: () => void;
    dismiss?: () => void;
    variant?: "default" | "destructive";
    size?: "sm" | "md" | "lg";
  }

  const props = withDefaults(defineProps<Props>(), {
    error: null,
    title: "Error",
    variant: "destructive",
    size: "md",
  });

  const errorMessage = computed(() => {
    if (!props.error) return "An unknown error occurred";

    if (typeof props.error === "string") {
      return props.error;
    }

    if ("message" in props.error) {
      return props.error.message;
    }

    return "An unknown error occurred";
  });

  const errorCode = computed(() => {
    if (
      props.error &&
      typeof props.error === "object" &&
      "code" in props.error
    ) {
      return props.error.code;
    }
    return null;
  });

  const sizeClasses = computed(() => {
    switch (props.size) {
      case "sm":
        return "text-xs p-2";
      case "lg":
        return "text-base p-4";
      default:
        return "text-sm p-3";
    }
  });

  const iconSize = computed(() => {
    switch (props.size) {
      case "sm":
        return 14;
      case "lg":
        return 20;
      default:
        return 16;
    }
  });
</script>

<template>
  <Alert v-if="error" :variant="variant" :class="['w-full', sizeClasses]">
    <AlertCircle :size="iconSize" class="shrink-0" />

    <div class="flex-1 min-w-0">
      <AlertTitle class="mb-1 flex items-center justify-between">
        <span>{{ title }}</span>
        <Button
          v-if="dismiss"
          variant="ghost"
          size="icon"
          class="h-5 w-5 -mr-2"
          @click="dismiss"
        >
          <X :size="iconSize" />
        </Button>
      </AlertTitle>

      <AlertDescription>
        {{ errorMessage }}
        <span v-if="errorCode" class="block mt-1 text-xs opacity-70">
          Code: {{ errorCode }}
        </span>
      </AlertDescription>
    </div>

    <Button
      v-if="retry"
      variant="outline"
      size="sm"
      class="mt-2 w-full sm:w-auto sm:ml-auto sm:mt-0"
      @click="retry"
    >
      <RefreshCw :size="iconSize" class="mr-2" />
      Retry
    </Button>
  </Alert>
</template>

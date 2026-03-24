<script setup lang="ts">
  import ToggleFavoriteButton from "@/components/shared/ToggleFavoriteButton.vue";
  import { ICommand } from "@/lib/api/api.types.ts";
  import { useToggleFavoriteCommand } from "@/lib/api/composables/commands.ts";

  defineProps<{
    command: ICommand;
    hideDescription?: boolean;
  }>();

  const { mutate: toggleFavoriteCommand } = useToggleFavoriteCommand();
</script>

<template>
  <div class="flex items-center gap-1 min-w-0">
    <ToggleFavoriteButton
      v-if="command"
      :isFavorite="command.isFavorite"
      @toggleFavorite="() => toggleFavoriteCommand({id: command.id})"
    />
    <div class="flex flex-col min-w-0">
      <span class="font-semibold text-md truncate"> {{ command.name }} </span>
      <span
        v-if="command.description && !hideDescription"
        class="text-xs tracking-wide text-muted-foreground truncate"
      >
        {{ command.description }}
      </span>
    </div>
  </div>
</template>

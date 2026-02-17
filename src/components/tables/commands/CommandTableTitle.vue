<script setup lang="ts">
  import { ICommand } from "@/lib/api/api.types.ts";
  import { Button } from "@/components/ui/button";
  import { FilledStarIcon, StarIcon } from "@/assets/Icons.ts";
  import { useToggleFavoriteCommand } from "@/lib/api/composables/commands.ts";

  defineProps<{
    command: ICommand;
  }>();

  const { mutate: toggleFavoriteCommand } = useToggleFavoriteCommand();
</script>

<template>
  <div class="flex items-center gap-3 min-w-0">
    <Button
      variant="ghost"
      size="icon"
      class="h-8 w-8 shrink-0 text-foreground"
      @click="toggleFavoriteCommand({id: command.id})"
    >
      <FilledStarIcon
        class="text-yellow-500 favorite-button"
        v-if="command.isFavorite"
      />
      <StarIcon class="favorite-button" v-else />
    </Button>
    <div class="flex flex-col min-w-0">
      <span class="font-semibold mb-1 text-base truncate">
        {{ command.name }}
      </span>
      <span
        v-if="command.description"
        class="text-xs tracking-wide text-muted-foreground truncate"
        ,
      >
        {{ command.description }}
      </span>
    </div>
  </div>
</template>

<style scoped>
  .favorite-button {
    width: 20px;
    height: 20px;
  }
</style>

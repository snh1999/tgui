<script setup lang="ts">
  import { Button } from "@/components/ui/button";
  import { Star } from "lucide-vue-next";

  defineProps<{
    isFavorite?: boolean;
  }>();

  const emit = defineEmits<{
    toggleFavorite: [];
  }>();
</script>

<template>
  <Button
    variant="ghost"
    size="icon"
    :class="['h-7 w-7', isFavorite ? 'text-yellow-500' : 'text-muted-foreground']"
    @click="emit('toggleFavorite')"
    :title="isFavorite ? 'Unfavorite' : 'Favorite' "
  >
    <Star :class="['h-4 w-4', isFavorite ? 'fill-current' : '']" />
  </Button>
</template>

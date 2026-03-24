<script setup lang="ts">
  import { FolderIcon } from "lucide-vue-next";
  import { computed, ref } from "vue";
  import CategoryBadge from "@/components/categories/CategoryBadge.vue";
  import CommandMenu from "@/components/commands/CommandMenu.vue";
  import CommandText from "@/components/commands/CommandText.vue";
  import StatusDot from "@/components/commands/StatusDot.vue";
  import ToggleFavoriteButton from "@/components/shared/ToggleFavoriteButton.vue";
  import {
    Card,
    CardAction,
    CardContent,
    CardFooter,
    CardHeader,
  } from "@/components/ui/card";
  import { ICommandWithHistory } from "@/lib/api/api.types.ts";
  import { useToggleFavoriteCommand } from "@/lib/api/composables/commands.ts";

  const props = defineProps<{
    command: ICommandWithHistory;
    isCard?: boolean;
  }>();

  const hovered = ref(false);
  const showHoverMenu = computed(
    () =>
      hovered.value ||
      (props.command.history?.status &&
        (props.command.history.status === "running" ||
          props.command.history.status === "failed"))
  );

  const { mutate: toggleFavoriteCommand } = useToggleFavoriteCommand();
</script>

<template>
  <Card
    class="w-full mb-2 transition-all duration-200 ease-out"
    :class="{
      'bg-muted/5': hovered,
    }"
    @mouseenter="hovered = true"
    @mouseleave="hovered = false"
  >
    <CardHeader class="flex flex-col px-4 py-1">
      <div class="flex w-full justify-between items-center gap-2.5 min-w-0">
        <ToggleFavoriteButton
          v-if="command"
          :isFavorite="command.isFavorite"
          @toggleFavorite="() => toggleFavoriteCommand({id: command.id})"
        />

        <CategoryBadge
          v-if="command.categoryId"
          :categoryId="command.categoryId"
        />
      </div>
      <div class="flex w-full justify-between">
        <div class="flex px-2 py-0.5 items-center gap-2.5 min-w-0">
          <span class="font-semibold text-md truncate">
            {{ command.name }}
          </span>
        </div>

        <CardAction v-show="!isCard && showHoverMenu">
          <CommandMenu :id="command.id" :status="command.history?.status" />
        </CardAction>
      </div>

      <p
        class="text-xs text-muted-foreground truncate ml-2 pl-2 border-l-2 border-border"
        :class="{'italic opacity-60': !command.description}"
      >
        {{ command.description ?? 'No description provided' }}
      </p>
    </CardHeader>

    <CardContent class="flex flex-col gap-1 pt-0">
      <CommandText :command="command" />

      <div class="flex py-1 justify-between items-center gap-3 flex-wrap">
        <span
          class="flex items-center gap-1.5 text-[11px] text-muted-foreground"
          :class="{'italic opacity-60': !command.description}"
        >
          <FolderIcon :size="13" class="opacity-50" />
          <span class="max-w-50 truncate">
            {{ command.workingDirectory ?? 'Directory not set' }}
          </span>
        </span>
        <StatusDot
          :status="command.history?.status"
          :pid="command.history?.pid"
        />
      </div>
    </CardContent>

    <CardFooter v-show="isCard" class="flex items-end justify-end opacity-85">
      <CommandMenu :id="command.id" :status="command.history?.status" />
    </CardFooter>
  </Card>
</template>

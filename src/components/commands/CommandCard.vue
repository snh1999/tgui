<script setup lang="ts">
  import { useTimeAgo } from "@vueuse/core";
  import { Container, FolderIcon, LucideClock } from "lucide-vue-next";
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

  const envVarCount = computed(
    () => Object.keys(props.command.envVars ?? {}).length
  );

  const timeAgo = useTimeAgo(() => props.command.history?.startedAt ?? "");

  const { mutate: toggleFavoriteCommand } = useToggleFavoriteCommand();
</script>

<template>
  <Card
    class="w-full px-0 pb-0 mb-2 transition-all duration-200 ease-out"
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

        <span>
          <StatusDot
            v-show="false"
            :status="command.history?.status"
            :pid="command.history?.pid"
          />
          <CategoryBadge
            v-if="command.categoryId"
            :categoryId="command.categoryId"
          />
        </span>
      </div>
      <div class="flex w-full justify-between">
        <span class="px-2 font-semibold text-md truncate">
          {{ command.name }}
        </span>

        <CardAction v-show="!isCard && showHoverMenu">
          <CommandMenu :id="command.id" :status="command.history?.status" />
        </CardAction>
      </div>

      <p
        class="text-xs text-muted-foreground truncate ml-2 border-border"
        :class="{'italic opacity-60': !command.description}"
      >
        {{ command.description ?? 'No description provided' }}
      </p>
    </CardHeader>

    <CardContent class="flex flex-col gap-1 pt-0">
      <CommandText :command="command" />

      <div
        class="flex text-[11px] pt-1 justify-between items-center gap-3 flex-wrap"
      >
        <span
          class="flex items-center gap-1.5  text-muted-foreground"
          :class="{'italic opacity-60': !command.description}"
        >
          <FolderIcon :size="13" class="opacity-50" />
          <span class="max-w-50 truncate text-muted-foreground/80">
            {{ command.workingDirectory ?? 'Directory not set' }}
          </span>
        </span>

        <span v-if="envVarCount > 0" class="env-count">
          <Container :size="14" class="opacity-50" />
          <span class="opacity-70 font-bold">{{ envVarCount }}</span>
          <span class="opacity-60"
            >env vars{{ envVarCount !== 1 ? "s" : "" }}</span
          >
        </span>
      </div>
    </CardContent>

    <CardFooter class="flex bg-muted/50 items-center justify-between py-2">
      <div class="flex items-center gap-3 text-[11px] text-muted-foreground">
        <span
          v-if="command.history?.startedAt"
          class="flex gap-1 text-muted-foreground/70"
        >
          <LucideClock :size="14" />
          {{ timeAgo }}
        </span>
        <span v-else class="opacity-50">Never run</span>
      </div>

      <CommandMenu
        v-if="isCard"
        :id="command.id"
        :status="command.history?.status"
      />
    </CardFooter>
  </Card>
</template>

<style scoped>
  .card-title {
    font-size: 0.9375rem;
    font-weight: 600;
    color: var(--foreground);
    line-height: 1.3;
    margin: 0;
  }

  .description-text {
    font-size: 0.75rem;
    color: var(--muted-foreground);
    line-height: 1.4;
    margin: 0.25rem 0 0;
    padding-left: 0.75rem;
    border-left: 2px solid var(--border);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .command-preview {
    display: flex;
    align-items: stretch;
    background: var(--muted);
    border-radius: 6px;
    overflow: hidden;
  }

  .shell-name {
    font-size: 0.6875rem;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.025em;
    opacity: 0.6;
  }

  .command-text {
    flex: 1;
    padding: 0.5rem 0.625rem;
    font-family:
      ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.75rem;
    color: var(--foreground);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .work-dir {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.6875rem;
    color: var(--muted-foreground);
    padding: 0 0.25rem;
  }

  .dir-prefix {
    opacity: 0.5;
    font-weight: 500;
  }

  .dir-path {
    opacity: 0.7;
    min-width: 0;
  }

  .env-count {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .time-ago {
    opacity: 0.8;
  }
</style>

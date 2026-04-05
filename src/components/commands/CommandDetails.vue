<script setup lang="ts">
  import { useDateFormat, useTimeAgo } from "@vueuse/core";
  import {
    ChevronDown,
    FolderOpen,
    Globe2,
    KeyRound,
    Terminal,
  } from "lucide-vue-next";
  import { computed, ref } from "vue";
  import { Badge } from "@/components/ui/badge";
  import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
  } from "@/components/ui/collapsible";

  import { ScrollArea } from "@/components/ui/scroll-area";
  import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
  } from "@/components/ui/tooltip";
  import { ICommand } from "@/lib/api/api.types.ts";
  import { useFormatDateTime } from "@/lib/utils.ts";

  const props = defineProps<{
    command: ICommand;
  }>();

  const commandPreview = computed(() =>
    props.command
      ? [props.command.command, ...(props.command.arguments ?? [])].join(" ")
      : ""
  );

  const envVars = computed(() => Object.entries(props.command.envVars ?? {}));

  const envOpen = ref(false);
</script>

<template>
  <TooltipProvider :delay-duration="400">
    <div class="flex flex-col h-full min-h-0">
      <ScrollArea v-if="command" class="h-full">
        <div class="px-5 py-4 space-y-5">
          <p v-if="command.description" class="text-sm text-muted-foreground">
            {{ command.description }}
          </p>
          <div class="rounded-md border border-border divide-y divide-border">
            <div class="flex items-start gap-3 px-3 py-2.5">
              <Terminal
                class="h-3.5 w-3.5 mt-0.5 text-muted-foreground shrink-0"
              />
              <div class="min-w-0 flex-1">
                <p class="text-[10px] text-muted-foreground mb-0.5">Command</p>
                <code class="font-mono text-xs break-all">
                  {{ commandPreview }}
                </code>
              </div>
            </div>

            <div
              v-if="command.arguments?.length"
              class="flex items-start gap-3 px-3 py-2.5"
            >
              <span
                class="text-[10px] font-mono text-muted-foreground mt-0.5 shrink-0 w-3.5 text-center"
                >#</span
              >
              <div>
                <p class="text-[10px] text-muted-foreground mb-1">Arguments</p>
                <div class="flex flex-wrap gap-1">
                  <Badge
                    v-for="arg in command.arguments"
                    :key="arg"
                    variant="secondary"
                    class="font-mono text-[10px] px-1.5 py-0 h-4"
                  >
                    {{ arg }}
                  </Badge>
                </div>
              </div>
            </div>

            <div
              v-if="command.workingDirectory"
              class="flex items-start gap-3 px-3 py-2.5"
            >
              <FolderOpen
                class="h-3.5 w-3.5 mt-0.5 text-muted-foreground shrink-0"
              />
              <div class="min-w-0">
                <p class="text-[10px] text-muted-foreground mb-0.5">
                  Working Directory
                </p>
                <code class="font-mono text-xs break-all text-muted-foreground">
                  {{ command.workingDirectory }}
                </code>
              </div>
            </div>

            <div class="flex items-start gap-3 px-3 py-2.5">
              <Globe2
                class="h-3.5 w-3.5 mt-0.5 text-muted-foreground shrink-0"
              />
              <div>
                <p class="text-[10px] text-muted-foreground mb-0.5">Shell</p>
                <span class="text-xs"
                  >{{ command.shell ?? 'System default' }}</span
                >
              </div>
            </div>

            <Collapsible v-if="envVars.length" v-model:open="envOpen">
              <CollapsibleTrigger
                class="flex items-center gap-3 px-3 py-2.5 w-full hover:bg-muted/40 transition-colors"
              >
                <KeyRound class="h-3.5 w-3.5 text-muted-foreground shrink-0" />
                <div class="flex-1 text-left">
                  <p class="text-[10px] text-muted-foreground">
                    Environment Variables
                  </p>
                  <p class="text-xs mt-0.5">
                    {{ envVars.length }} variable
                    {{ envVars.length !== 1 ? 's' : '' }}
                  </p>
                </div>
                <ChevronDown
                  :class="['h-3.5 w-3.5 text-muted-foreground transition-transform', envOpen ? 'rotate-180' : '']"
                />
              </CollapsibleTrigger>
              <CollapsibleContent>
                <div class="px-3 pb-2.5">
                  <div
                    class="rounded-sm border border-border divide-y divide-border/50 overflow-hidden"
                  >
                    <div
                      v-for="[key, val] in envVars"
                      :key="key"
                      class="flex items-baseline gap-2 px-2.5 py-1.5 text-xs"
                    >
                      <code
                        class="font-mono text-muted-foreground shrink-0 min-w-25"
                      >
                        {{ key }}
                      </code>
                      <code class="font-mono truncate text-foreground/80">
                        {{ val || '(empty)' }}
                      </code>
                    </div>
                  </div>
                </div>
              </CollapsibleContent>
            </Collapsible>
          </div>

          <div
            class="flex items-center justify-between text-xs text-muted-foreground"
          >
            <Tooltip>
              <TooltipTrigger as-child>
                <span class="cursor-default"
                  >Created {{ useTimeAgo(command.createdAt??"") }}</span
                >
              </TooltipTrigger>
              <TooltipContent>
                {{ useFormatDateTime(command.createdAt) }}
              </TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger as-child>
                <span class="cursor-default"
                  >Updated {{ useTimeAgo(command.updatedAt??"") }}</span
                >
              </TooltipTrigger>
              <TooltipContent>
                {{ useFormatDateTime(command.updatedAt) }}
              </TooltipContent>
            </Tooltip>
          </div>
        </div>
      </ScrollArea>
    </div>
  </TooltipProvider>
</template>

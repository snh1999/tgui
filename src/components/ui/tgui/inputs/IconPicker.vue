<script setup lang="ts">
  import { computed, ref, watch } from "vue";
  import * as Icons from "lucide-vue-next";
  import { Button } from "@/components/ui/button";
  import { Input } from "@/components/ui/input";
  import {
    Popover,
    PopoverContent,
    PopoverTrigger,
  } from "@/components/ui/popover";
  import {
    InputGroup,
    InputGroupAddon,
    InputGroupButton,
    InputGroupInput,
  } from "@/components/ui/input-group";

  const props = defineProps<{
    modelValue?: string;
    id?: string;
    name?: string;
    placeholder?: string;
    disabled?: boolean;
  }>();

  const emit = defineEmits<{
    "update:modelValue": [value: string];
  }>();

  const ALL_ICONS: string[] = Object.keys(Icons).filter(
    (key) =>
      key !== "createLucideIcon" &&
      key !== "default" &&
      key !== "icons" &&
      key !== "Icon" &&
      key !== "IconNode" &&
      key !== "LucideIcon" &&
      key !== "LucideProps" &&
      key !== "SVGProps"
  );

  const PAGE_SIZE = 80;

  const search = ref("");
  const open = ref(false);
  const page = ref(1);

  // Match icon name without case, also support space-separated words ("arrow left")
  const filteredIcons = computed(() => {
    const q = search.value.trim().toLowerCase().replace(/\s+/g, "");
    if (!q) return ALL_ICONS;
    return ALL_ICONS.filter((name) => name.toLowerCase().includes(q));
  });

  const paginatedIcons = computed(() =>
    filteredIcons.value.slice(0, page.value * PAGE_SIZE)
  );

  const hasMore = computed(
    () => paginatedIcons.value.length < filteredIcons.value.length
  );

  // Reset pagination when search changes
  watch(search, () => {
    page.value = 1;
  });

  function getIconComponent(name: string) {
    return (Icons as Record<string, unknown>)[name] as object;
  }

  const model = computed({
    get: () => props.modelValue || "",
    set: (v) => emit("update:modelValue", v),
  });

  function select(name: string) {
    emit("update:modelValue", name);
    open.value = false;
    search.value = "";
  }

  function clear() {
    emit("update:modelValue", "");
  }
</script>

<template>
  <Popover v-model:open="open">
    <InputGroup class="h-10">
      <InputGroupInput
        :id="id"
        :name="name"
        v-model="model"
        :placeholder="placeholder || 'Select a directory...'"
        readonly
        :disabled="disabled"
        class="flex-1 truncate"
        :class="{ 'text-muted-foreground': !modelValue }"
      />

      <InputGroupAddon>
        <component
          :is="getIconComponent(modelValue!)"
          v-if="modelValue"
          :size="16"
          class="shrink-0"
        />
        <Icons.SquareMousePointer
          v-else
          :size="16"
          class="shrink-0 opacity-70"
        />
      </InputGroupAddon>

      <InputGroupAddon align="inline-end">
        <InputGroupButton
          v-if="modelValue && !disabled"
          variant="ghost"
          @click.stop="clear"
        >
          <Icons.X />
        </InputGroupButton>
        <PopoverTrigger as-child :disabled="disabled">
          <InputGroupButton
            type="button"
            size="sm"
            :disabled="disabled"
            class="shrink-0"
            variant="ghost"
          >
            <Icons.ArrowUp />
            {{ modelValue ? 'Change' : 'Select' }}
          </InputGroupButton>
        </PopoverTrigger>
      </InputGroupAddon>
    </InputGroup>

    <PopoverContent class="w-85 p-0" align="start" :side-offset="4">
      <div class="border-b p-2">
        <div class="relative">
          <Icons.Search
            :size="14"
            class="absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none"
          />
          <Input
            v-model="search"
            placeholder="Search icons..."
            class="pl-8 h-8 text-sm"
            autofocus
          />
        </div>
        <p class="text-xs text-muted-foreground mt-1.5 px-0.5">
          {{ filteredIcons.length }} icon
          {{ filteredIcons.length !== 1 ? 's' : '' }}
        </p>
      </div>

      <div class="overflow-y-auto max-h-70 p-2">
        <div
          v-if="filteredIcons.length === 0"
          class="py-8 text-center text-sm text-muted-foreground"
        >
          No icons found for "{{ search }}"
        </div>

        <div class="grid grid-cols-8 gap-0.5">
          <button
            v-for="iconName in paginatedIcons"
            :key="iconName"
            type="button"
            :title="iconName"
            class="flex items-center justify-center rounded-md p-1.5 hover:bg-accent hover:text-accent-foreground transition-colors"
            :class="{
              'bg-accent text-accent-foreground ring-1 ring-ring': modelValue === iconName,
            }"
            @click="select(iconName)"
          >
            <component
              :is="getIconComponent(iconName)"
              :size="18"
              :stroke-width="1.5"
            />
          </button>
        </div>

        <div v-if="hasMore" class="mt-2 flex justify-center">
          <Button
            type="button"
            variant="ghost"
            size="sm"
            class="text-xs w-full"
            @click="page++"
          >
            Load more ({{ filteredIcons.length - paginatedIcons.length }}
            remaining)
          </Button>
        </div>
      </div>

      <div
        v-if="modelValue"
        class="border-t px-3 py-2 flex items-center gap-2 text-xs text-muted-foreground"
      >
        <component :is="getIconComponent(modelValue)" :size="16" />
        <span class="font-mono">{{ modelValue }}</span>
        <button
          type="button"
          class="ml-auto hover:text-foreground transition-colors"
          @click="clear"
        >
          Clear
        </button>
      </div>
    </PopoverContent>
  </Popover>
</template>

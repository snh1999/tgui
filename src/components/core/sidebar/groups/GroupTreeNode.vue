<script setup lang="ts">
  import { ChevronRight, ExternalLink } from "lucide-vue-next";
  import { ComputedRef, computed, inject, ref } from "vue";
  import { useRouter } from "vue-router";
  import GroupCategoryLine from "@/components/shared/GroupCategoryLine.vue";
  import { Button } from "@/components/ui/button";
  import { Collapsible, CollapsibleContent } from "@/components/ui/collapsible";
  import {
    SidebarMenuButton,
    SidebarMenuItem,
    SidebarMenuSub,
  } from "@/components/ui/sidebar";
  import { IGroup } from "@/lib/api/api.types.ts";
  import { useGetGroupTree } from "@/lib/api/composables/groups.ts";

  const router = useRouter();

  const props = defineProps<{
    group: IGroup;
    showLinkButton?: boolean;
  }>();

  const onGroupSelected = inject<(id: number) => void>("onGroupSelected");
  const selectedGroupId = inject<ComputedRef<number>>("selectedGroupId");

  const isOpen = ref(false);
  const hasExpanded = ref(false);

  const { data: tree, isLoading } = useGetGroupTree(
    computed(() => (hasExpanded.value ? props.group.id : 0))
  );

  const children = computed(() => tree.value?.children ?? []);

  function toggle() {
    if (!hasExpanded.value) {
      hasExpanded.value = true;
    }
    isOpen.value = !isOpen.value;
  }
</script>

<template>
  <SidebarMenuItem>
    <Collapsible v-model:open="isOpen">
      <div class="flex items-center gap-1">
        <Button
          variant="link"
          size="icon"
          class="w-6 h-6 shrink-0"
          @click.stop="toggle"
          :isPending="isLoading"
        >
          <ChevronRight
            v-if="!hasExpanded || children.length > 0"
            class="w-4 h-4 transition-transform duration-200"
            :class="{ 'rotate-90': isOpen }"
          />
        </Button>

        <SidebarMenuButton
          class="flex-1"
          @click="onGroupSelected?.(group.id)"
          :isActive="selectedGroupId === group.id"
        >
          <GroupCategoryLine :element="group" />
        </SidebarMenuButton>

        <Button
          v-if="showLinkButton"
          variant="ghost"
          size="icon-sm"
          class="w-6 h-6 shrink-0"
          @click="router.push(`/groups/${group.id}`)"
        >
          <ExternalLink />
        </Button>
      </div>

      <CollapsibleContent>
        <SidebarMenuSub>
          <GroupTreeNode
            v-for="child in children"
            :key="child.group.id"
            :group="child.group"
          />
        </SidebarMenuSub>
      </CollapsibleContent>
    </Collapsible>
  </SidebarMenuItem>
</template>

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
    useSidebar,
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
  const sidebar = useSidebar();

  function toggle() {
    if (!hasExpanded.value) {
      hasExpanded.value = true;
    }
    isOpen.value = !isOpen.value;
  }

  const isCollapsed = computed(() => sidebar.state.value === "collapsed");
</script>

<template>
  <SidebarMenuItem>
    <Collapsible v-model:open="isOpen">
      <SidebarMenuButton asChild :isActive="selectedGroupId === group.id">
        <div class="flex items-center gap-1">
          <Button
            v-show="!isCollapsed"
            variant="link"
            size="icon"
            class="w-4 h-6 shrink-0 text-secondary hover:scale-125"
            @click.stop="toggle"
            :isPending="isLoading"
          >
            <ChevronRight
              v-if="!hasExpanded || children.length > 0"
              class="w-4 h-4 transition-transform duration-200"
              :class="{ 'rotate-90': isOpen }"
            />
          </Button>
          <span class="flex-1" @click="onGroupSelected?.(group.id)">
            <GroupCategoryLine :element="group" />
          </span>

          <Button
            v-if="showLinkButton && !isCollapsed"
            variant="link"
            size="icon-sm"
            class="w-4 h-6 shrink-0 text-secondary hover:scale-120"
            @click="router.push(`/groups/${group.id}`)"
          >
            <ExternalLink />
          </Button>
        </div>
      </SidebarMenuButton>

      <CollapsibleContent>
        <SidebarMenuSub>
          <GroupTreeNode
            v-for="child in children"
            :key="child.group.id"
            :group="child.group"
            :showLinkButton="showLinkButton"
          />
        </SidebarMenuSub>
      </CollapsibleContent>
    </Collapsible>
  </SidebarMenuItem>
</template>

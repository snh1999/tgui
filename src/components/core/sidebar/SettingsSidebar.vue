<script setup lang="ts">
  import { ChevronLeft } from "lucide-vue-next";
  import { capitalize } from "vue";
  import { useRouter } from "vue-router";
  import { Button } from "@/components/ui/button";
  import {
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
  } from "@/components/ui/sidebar";
  import {
    SETTINGS_SECTION,
    TSettingsSection,
    useSettingsStateStore,
  } from "@/stores/settings.store";

  const settingsStore = useSettingsStateStore();
  const router = useRouter();

  const scrollToSection = (sectionId: TSettingsSection) => {
    const element = document.getElementById(sectionId);
    if (element) {
      element.scrollIntoView({ behavior: "smooth", block: "start" });
      settingsStore.setSelectedSection(sectionId);
    }
  };
</script>

<template>
  <SidebarMenu class="sticky top-0">
    <Button @click="router.back()" class="w-max border-none" variant="ghost">
      <ChevronLeft class="w-4 h-4" />
    </Button>

    <SidebarMenuItem
      v-for="section in SETTINGS_SECTION"
      :key="section"
      class="px-2"
    >
      <SidebarMenuButton
        @click="scrollToSection(section)"
        :is-active="section === settingsStore.selectedSection"
        class="w-full justify-start"
      >
        <span class="text-sm">{{ capitalize(section) }}</span>
      </SidebarMenuButton>
    </SidebarMenuItem>
  </SidebarMenu>
</template>

<script setup lang="ts">
  /** biome-ignore-all lint/complexity/noForEach: <for simplicity> */

  import { type Component, onMounted, onUnmounted } from "vue";
  import AppearanceSettings from "@/components/settings/sections/AppearanceSettings.vue";
  import DataAndLogsSettings from "@/components/settings/sections/DataAndLogsSettings.vue";
  import ExecutionSettings from "@/components/settings/sections/ExecutionSettings.vue";
  import SettingsDangerZone from "@/components/settings/sections/SettingsDangerZone.vue";
  import {
    TSettingsSection,
    useSettingsStateStore,
  } from "@/stores/settings.store";

  const settingsStore = useSettingsStateStore();

  const componentMap: Record<TSettingsSection, Component> = {
    appearance: AppearanceSettings,
    execution: ExecutionSettings,
    logs: DataAndLogsSettings,
    "danger zone": SettingsDangerZone,
  };
  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          settingsStore.setSelectedSection(entry.target.id as TSettingsSection);
        }
      });
    },
    { threshold: 0.3, rootMargin: "-10% 0px -60% 0px" }
  );

  onMounted(() => {
    document.querySelectorAll("section[data-section]").forEach((section) => {
      observer.observe(section);
    });
  });

  onUnmounted(() => observer.disconnect());
</script>

<template>
  <div class="flex w-full h-full">
    <div class="flex-1 overflow-y-auto scroll-smooth">
      <div class="w-full p-10 space-y-10 pb-[70vh]">
        <section
          v-for="(component, path) in componentMap"
          :id="path"
          data-section
          class="scroll-mt-10"
        >
          <Component :is="component" />
        </section>
      </div>
    </div>
  </div>
</template>

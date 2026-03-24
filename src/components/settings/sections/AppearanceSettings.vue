<script setup lang="ts">
  import { ref, watch } from "vue";
  import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from "@/components/ui/select";
  import SettingsSectionWrapper from "@/components/settings/SettingsSectionWrapper.vue";
  import SettingsRow from "@/components/settings/SettingsRow.vue";

  const STORAGE_KEY = "appearance:theme";

  const theme = ref<string>(localStorage.getItem(STORAGE_KEY) ?? "system");

  watch(theme, (v) => {
    localStorage.setItem(STORAGE_KEY, v);
  });
</script>

<template>
  <SettingsSectionWrapper
    title="Appearance"
    description="Customize how the application looks and feels."
  >
    <SettingsRow
      label="Theme"
      description="Controls the color scheme of the application."
    >
      <Select
        :model-value="theme"
        @update:model-value="(v) => theme = v as string"
      >
        <SelectTrigger class="settings-select">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="system">System</SelectItem>
          <SelectItem value="light">Light</SelectItem>
          <SelectItem value="dark">Dark</SelectItem>
        </SelectContent>
      </Select>
    </SettingsRow>
  </SettingsSectionWrapper>
</template>

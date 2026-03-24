import { defineStore } from "pinia";
import { ref } from "vue";

export const SETTINGS_SECTION = [
  "appearance",
  "execution",
  "logs",
  "danger zone",
] as const;

export type TSettingsSection = (typeof SETTINGS_SECTION)[number];

export const useSettingsStateStore = defineStore("settingsState", () => {
  const selectedSection = ref<TSettingsSection | null>();
  function setSelectedSection(newSelection: TSettingsSection | null) {
    selectedSection.value = newSelection;
    console.log(newSelection, selectedSection.value);
  }
  return {
    selectedSection,
    setSelectedSection,
  };
});

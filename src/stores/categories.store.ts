import { defineStore } from "pinia";
import { ref } from "vue";
import type { TViewMode } from "@/stores/commands.store.ts";

export const useCategoryStore = defineStore("categoriesState", () => {
  const selectedCategory = ref<number | null>(null);
  const commandsView = ref<TViewMode>("list");
  function setCommandsView(viewMode: TViewMode) {
    commandsView.value = viewMode;
  }

  const groupsView = ref<TViewMode>("list");
  function setGroupsView(viewMode: TViewMode) {
    groupsView.value = viewMode;
  }

  function setSelectedCategory(category: null | number) {
    selectedCategory.value = category;
  }
  return {
    selectedCategory,
    setSelectedCategory,
    commandsView,
    setCommandsView,
    groupsView,
    setGroupsView,
  };
});

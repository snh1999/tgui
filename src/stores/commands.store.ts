import { defineStore } from "pinia";
import { computed, ref } from "vue";

export type TViewMode = "list" | "grid" | "table";

export const useCommandsStore = defineStore("commandsStore", () => {
  const search = ref("");
  const selectedGroup = ref<number | "none">("none");
  const filterCategory = ref<number | "all">("all");
  const favoritesOnly = ref(false);
  const showRunningOnly = ref(false);
  const view = ref<TViewMode>("list");

  const isFilterChanged = computed(
    () =>
      favoritesOnly.value !== false ||
      filterCategory.value !== "all" ||
      showRunningOnly.value !== false
  );

  function clearFilter() {
    favoritesOnly.value = false;
    filterCategory.value = "all";
    showRunningOnly.value = false;
  }

  return {
    search,
    filterCategory,
    showFavoritesOnly: favoritesOnly,
    showRunningOnly,
    view,
    selectedGroup,
    clearFilter,
    isFilterChanged,
  };
});

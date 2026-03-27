import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { TViewMode } from "@/stores/commands.store.ts";

export const useGroupsStore = defineStore("groupsStore", () => {
  const search = ref("");
  const filterCategory = ref<number | "all">("all");
  const view = ref<TViewMode>("list");
  const favoritesOnly = ref<boolean>(false);

  const isFilterChanged = computed(
    () => favoritesOnly.value !== false || filterCategory.value !== "all"
  );

  function clearFilter() {
    favoritesOnly.value = false;
    filterCategory.value = "all";
  }

  return {
    search,
    filterCategory,
    view,
    favoritesOnly,
    isFilterChanged,
    clearFilter,
  };
});

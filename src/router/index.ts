import { createRouter, createWebHashHistory } from "vue-router";
import CategoryPage from "@/pages/CategoryPage.vue";
import CommandsPage from "@/pages/CommandsPage.vue";
import EmptyCategoriesPage from "@/pages/EmptyCategoriesPage.vue";
import SettingsPage from "@/pages/SettingsPage.vue";

export const routePaths = {
  home: "/",
  groups: "/groups",
  commands: "/commands",
  categories: "/categories",
  settings: "/settings",
};

const router = createRouter({
  history: createWebHashHistory(import.meta.env.BASE_URL),
  routes: [
    { path: routePaths.commands, name: "commands", component: CommandsPage },
    {
      path: routePaths.categories,
      name: "categories",
      component: EmptyCategoriesPage,
    },
    {
      path: `${routePaths.categories}/:id`,
      name: "category",
      component: CategoryPage,
    },
    { path: routePaths.settings, name: "settings", component: SettingsPage },
  ],
});

export default router;
export type TRoutePaths = keyof typeof routePaths;

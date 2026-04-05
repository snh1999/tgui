import { createRouter, createWebHashHistory } from "vue-router";
import CategoryPage from "@/pages/CategoryPage.vue";
import CommandPage from "@/pages/CommandPage.vue";
import CommandsPage from "@/pages/CommandsPage.vue";
import EmptyCategoriesPage from "@/pages/EmptyCategoriesPage.vue";
import GroupsPage from "@/pages/GroupsPage.vue";
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
      path: `${routePaths.commands}/:id`,
      name: "command",
      component: CommandPage,
    },
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
    { path: routePaths.groups, name: "groups", component: GroupsPage },
    { path: `${routePaths.groups}/:id`, name: "group", component: GroupsPage },
    { path: routePaths.settings, name: "settings", component: SettingsPage },
  ],
});
export default router;
export type TRoutePaths = keyof typeof routePaths;

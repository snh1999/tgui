import {
  createRouter,
  createWebHashHistory,
  type RouteLocationNormalizedGeneric,
} from "vue-router";
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

const validateIdParam = (
  to: RouteLocationNormalizedGeneric,
  redirectPath: string
) => {
  const id = Number(to.params.id);
  if (Number.isNaN(id) || id <= 0) {
    return { path: redirectPath, replace: true };
  }
};

const router = createRouter({
  history: createWebHashHistory(import.meta.env.BASE_URL),
  routes: [
    { path: routePaths.commands, name: "commands", component: CommandsPage },
    {
      path: `${routePaths.commands}/:id`,
      name: "command",
      component: CommandPage,
      beforeEnter: (to) => validateIdParam(to, routePaths.commands),
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
      beforeEnter: (to) => validateIdParam(to, routePaths.categories),
    },
    { path: routePaths.groups, name: "groups", component: GroupsPage },
    {
      path: `${routePaths.groups}/:id`,
      name: "group",
      component: GroupsPage,
      beforeEnter: (to) => validateIdParam(to, routePaths.groups),
    },
    { path: routePaths.settings, name: "settings", component: SettingsPage },
  ],
});
export default router;
export type TRoutePaths = keyof typeof routePaths;

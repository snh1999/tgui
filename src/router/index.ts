import { createRouter, createWebHashHistory } from "vue-router";
import BrowsePage from "@/pages/BrowsePage.vue";
import CommandPage from "@/pages/CommandPage.vue";
import CommandsPage from "@/pages/CommandsPage.vue";
import Dashboard from "@/pages/Dashboard.vue";
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
    { path: routePaths.home, name: "dashboard", component: Dashboard },
    { path: routePaths.groups, name: "groups", component: GroupsPage },
    { path: routePaths.commands, name: "commands", component: CommandsPage },
    {
      path: `${routePaths.commands}/:id`,
      name: "command-detail",
      component: CommandPage,
    },
    { path: "/browse", name: "browse", component: BrowsePage },
    { path: routePaths.settings, name: "settings", component: SettingsPage },
  ],
});

export default router;
export type TRoutePaths = keyof typeof routePaths;

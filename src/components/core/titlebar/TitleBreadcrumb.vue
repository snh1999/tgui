<script setup lang="ts">
  import { ChevronDownIcon } from "lucide-vue-next";
  import { capitalize, computed } from "vue";
  import { useRoute, useRouter } from "vue-router";
  import AppLogo from "@/components/core/AppLogo.vue";
  import {
    Breadcrumb,
    BreadcrumbItem,
    BreadcrumbLink,
    BreadcrumbList,
    BreadcrumbSeparator,
  } from "@/components/ui/breadcrumb";
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from "@/components/ui/dropdown-menu";
  import { routePaths, TRoutePaths } from "@/router";

  const route = useRoute();
  const router = useRouter();

  const routeLabels: Record<TRoutePaths, string> = {
    home: "Home",
    groups: "Groups",
    commands: "Commands",
    categories: "Categories",
    settings: "Settings",
  };

  const breadcrumbText = computed(() => {
    const name = route.name as TRoutePaths;
    return routeLabels[name] || capitalize(name) || "Home";
  });
</script>

<template>
  <Breadcrumb class="pl-2">
    <BreadcrumbList>
      <BreadcrumbItem>
        <BreadcrumbLink as-child>
          <RouterLink to="/">
            <AppLogo :scale="0.8" />
          </RouterLink>
        </BreadcrumbLink>
      </BreadcrumbItem>
      <BreadcrumbSeparator />
      <BreadcrumbItem>
        <DropdownMenu>
          <DropdownMenuTrigger
            class="flex items-center gap-1 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*=\'size-\'])]:size-3.5"
          >
            {{ breadcrumbText }}
            <ChevronDownIcon />
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start">
            <DropdownMenuItem
              v-for="(label, path) in routeLabels"
              :key="path"
              @click="router.push(routePaths[path])"
            >
              {{ label }}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </BreadcrumbItem>
    </BreadcrumbList>
  </Breadcrumb>
</template>

<script setup lang="ts">
  import { ChevronDownIcon } from "lucide-vue-next";
  import { computed } from "vue";
  import { useRoute, useRouter } from "vue-router";
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

  const breadcrumb = computed(() => {
    const name = route.name as TRoutePaths;
    return {
      app: "TGUI",
      current: routeLabels[name] || name || "Home",
    };
  });
</script>

<template>
  <Breadcrumb class="pl-2">
    <BreadcrumbList>
			<BreadcrumbItem>
				<BreadcrumbLink as-child>
					<a href="/">{{ breadcrumb.app }}</a>
				</BreadcrumbLink>
			</BreadcrumbItem>
			<BreadcrumbSeparator/>
			<BreadcrumbItem>
				<DropdownMenu>
					<DropdownMenuTrigger
							class="flex items-center gap-1 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*=\'size-\'])]:size-3.5"
					>
						{{ breadcrumb.current }}
						<ChevronDownIcon/>
					</DropdownMenuTrigger>
					<DropdownMenuContent align="start">
						<DropdownMenuItem
								v-for="(label, path) in routeLabels"
								:key="path"
								@click="router.push(routePaths[path])"
						>{{ label }}
						</DropdownMenuItem>
					</DropdownMenuContent>
				</DropdownMenu>
			</BreadcrumbItem>
		</BreadcrumbList>
	</Breadcrumb>
</template>

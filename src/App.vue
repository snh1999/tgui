<script setup lang="ts">
import AppSidebar from "@/components/core/sidebar/AppSidebar.vue";
import TitleBar from "@/components/core/titlebar/TitleBar.vue";
import { SidebarProvider } from "@/components/ui/sidebar";
import { Toaster } from "@/components/ui/sonner";
import DragHandle from "@/components/core/DragHandle.vue";
import { ref } from "vue";

const sidebarWidth = ref(256);
</script>

<template>
	<SidebarProvider
			storage-key="sidebar"
			:style="{ '--sidebar-width': sidebarWidth + 'px' }"
	>
		<TitleBar class="fixed top-0 left-0 right-0 z-50" />
		<div class="flex pt-10 h-screen w-screen">
			<AppSidebar />
			<DragHandle v-model="sidebarWidth" />
			<main class="flex-1 flex flex-col h-full overflow-auto bg-background text-foreground">
				<slot />
				<RouterView />
			</main>
		</div>
		<Toaster />
	</SidebarProvider>
</template>

<script setup lang="ts">
  import {
    Empty,
    EmptyContent,
    EmptyDescription,
    EmptyHeader,
    EmptyTitle,
  } from "@/components/ui/empty";
  import CreateCategoryDialog from "@/components/forms/categories/CreateCategoryDialog.vue";
</script>

<template>
  <Empty class="h-screen text-muted-foreground">
    <EmptyHeader>
      <EmptyTitle class="text-muted-foreground">
        No Categories Selected
      </EmptyTitle>
      <EmptyDescription>
        You haven't selected any category yet. Create a new category or Select
        an existing one from sidebar.
      </EmptyDescription>
    </EmptyHeader>
    <EmptyContent>
      <div class="flex gap-2">
        <CreateCategoryDialog
          viewTrigger
          triggerSize="lg"
          triggerVariant="primary"
        />
      </div>
    </EmptyContent>
  </Empty>
</template>

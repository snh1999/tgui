<script setup lang="ts">
  import { Copy } from "lucide-vue-next";
  import { ref } from "vue";
  import { useRouter } from "vue-router";
  import { DeleteIcon, EditIcon } from "@/assets/Icons.ts";
  import CategoryLine from "@/components/categories/CategoryLine.vue";
  import CreateCategoryDialog from "@/components/forms/categories/CreateCategoryDialog.vue";
  import UpdateCategoryDialog from "@/components/forms/categories/UpdateCategoryDialog.vue";
  import { Button } from "@/components/ui/button";
  import ConfirmDialog from "@/components/ui/tgui/ConfirmDialog.vue";
  import { ICategory } from "@/lib/api/api.types.ts";
  import { useDeleteCategory } from "@/lib/api/composables/categories.ts";

  const props = defineProps<{
    category: ICategory;
  }>();

  const emit = defineEmits<{
    delete: [id: number];
  }>();

  const router = useRouter();
  const { mutate: deleteCategory } = useDeleteCategory();

  const deleteDialogOpen = ref(false);

  function onDeleteClick() {
    deleteCategory(props.category.id);
    deleteDialogOpen.value = false;
    emit("delete", props.category.id);
    router.push("/categories");
  }
</script>

<template>
  <div
    class="flex items-center justify-between w-full h-12 px-4 border-b bg-card/50"
  >
    <div class="flex items-center gap-3">
      <CategoryLine
        :category="category"
        class="text-lg [&>span]:h-8 [&>span]:text-lg [&_svg]:size-6 [&_.rounded-md]:w-5 [&_.rounded-md]:h-5"
      />
    </div>

    <div class="flex items-center gap-2">
      <UpdateCategoryDialog
        :id="category.id"
        triggerVariant="outline"
        viewTrigger
      />

      <CreateCategoryDialog :category="category">
        <Copy class="h-3.5 w-3.5" />
        Duplicate
      </CreateCategoryDialog>

      <UpdateCategoryDialog :id="category.id">
        <template #trigger>
          <Button variant="ghost" size="sm">
            <EditIcon class="size-4" />
            <span class="ml-1">Edit</span>
          </Button>
        </template>
      </UpdateCategoryDialog>

      <Button
        variant="destructive"
        size="sm"
        title="Delete category"
        @click="deleteDialogOpen = true"
      >
        <DeleteIcon class="size-4" />
        <span class="ml-1">Delete</span>
      </Button>
    </div>

    <ConfirmDialog
      v-model:open="deleteDialogOpen"
      description="Are you sure you want to delete this category? Commands and groups will be moved to uncategorized."
      @confirm="onDeleteClick"
    />
  </div>
</template>

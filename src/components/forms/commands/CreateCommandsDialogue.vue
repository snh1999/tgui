<script setup lang="ts">
  import { ref, VNodeRef } from "vue";
  import { AddIcon } from "@/assets/Icons";
  import OpenDialog from "@/components/ui/tgui/OpenDialog.vue";
  import { Button } from "@/components/ui/button";
  import CreateCommandForm from "@/components/forms/commands/CreateCommandForm.vue";
  import { Field } from "@/components/ui/field";
  import { COMMAND_FORM_ID } from "@/app.constants.ts";

  const newCommandOpen = ref(false);
  const openNewCommand = () => (newCommandOpen.value = true);
  const closeNewCommand = () => (newCommandOpen.value = false);

  const createCommandFormRef = ref<VNodeRef | null>(null);
</script>

<template>
  <header class="header">
    <Button @click="openNewCommand" class="btn-primary gap-2">
      <AddIcon />
      New Command
    </Button>

    <OpenDialog v-model:open="newCommandOpen" title="Create New Command">
      <CreateCommandForm
        :onSuccess="closeNewCommand"
        ref="createCommandFormRef"
      />

      <template #footer>
        <Field orientation="horizontal">
          <Button
            type="button"
            variant="outline"
            @click="createCommandFormRef?.resetForm"
          >
            Reset
          </Button>
        </Field>

        <Button variant="outline" @click="closeNewCommand">Cancel</Button>
        <Button type="submit" :form="COMMAND_FORM_ID">Create</Button>
        <!--        <Button @click="newCommandOpen = false">Create</Button>-->
      </template>
    </OpenDialog>
  </header>
</template>

<script setup lang="ts">
import {ref} from "vue";
import {COMMAND_FORM_ID} from "@/app.constants.ts";
import {AddIcon} from "@/assets/Icons";
import CreateCommandForm from "@/components/forms/commands/CreateCommandForm.vue";
import {Button} from "@/components/ui/button";
import {Field} from "@/components/ui/field";
import OpenDialog from "@/components/ui/tgui/OpenDialog.vue";

const newCommandOpen = ref(false);
  const openNewCommand = () => {
    newCommandOpen.value = true;
  };
  const closeNewCommand = () => {
    newCommandOpen.value = false;
  };

  const createCommandFormRef = ref<InstanceType<
    typeof CreateCommandForm
  > | null>(null);
</script>

<template>
  <header class="header">
    <Button @click="openNewCommand" class="btn-primary gap-2">
      <AddIcon />
      New Command
    </Button>

    <OpenDialog
      class="min-w-[50%]"
      v-model:open="newCommandOpen"
      title="Create New Command"
    >
      <CreateCommandForm
        :onSuccess="closeNewCommand"
        ref="createCommandFormRef"
      />

      <template #footer>
        <Field orientation="horizontal">
          <Button
            type="button"
            variant="outline"
            @click="createCommandFormRef?.resetForm()"
          >
            Reset
          </Button>
        </Field>

        <Button variant="outline" @click="closeNewCommand">Cancel</Button>
        <Button type="submit" :form="COMMAND_FORM_ID">Create</Button>
      </template>
    </OpenDialog>
  </header>
</template>

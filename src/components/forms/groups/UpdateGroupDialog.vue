<script setup lang="ts">
  import { ref } from "vue";
  import { GROUP_FORM_ID } from "@/app.constants.ts";
  import { EditIcon } from "@/assets/Icons";
  import UpsertCommandForm from "@/components/forms/commands/UpsertCommandForm.vue";
  import { Button } from "@/components/ui/button";
  import ErrorDisplay from "@/components/ui/tgui/ErrorDisplay.vue";
  import Loading from "@/components/ui/tgui/Loading.vue";
  import { useGetGroup } from "@/lib/api/composables/groups.ts";
  import UpsertGroupForm from "@/components/forms/groups/UpsertGroupForm.vue";
  import FormDialog from "@/components/forms/common/FormDialog.vue";

  const props = defineProps<{
    id: number;
    viewTrigger?: boolean;
  }>();

  const {
    data: group,
    isPending,
    isError,
    error,
    refetch,
  } = useGetGroup(props.id);

  const updateGroupFormRef = ref<InstanceType<typeof UpsertGroupForm>>();
</script>

<template>
  <FormDialog title="Update Group">
    <template v-if="viewTrigger" #trigger>
      <EditIcon /> Edit Group
    </template>

    <template #default="{closeDialog}">
      <Loading v-if="isPending" />
      <ErrorDisplay v-if="isError" :error="error" :retry="refetch" />
      <UpsertCommandForm
        v-if="group"
        :key="group.id"
        :group="group"
        @success="closeDialog"
        ref="updateGroupFormRef"
      />
    </template>

    <template #reset>
      <Button
        variant="outline"
        @click="updateGroupFormRef?.resetForm()"
        :isPending="updateGroupFormRef?.isPending"
      >
        Reset
      </Button>
    </template>

    <template #submit>
      <Button
        type="submit"
        :form="GROUP_FORM_ID"
        :isPending="updateGroupFormRef?.isPending"
      >
        Update
      </Button>
    </template>
  </FormDialog>
</template>

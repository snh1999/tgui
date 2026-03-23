<script setup lang="ts">
  import { toast } from "vue-sonner";
  import { Button } from "@/components/ui/button";
  import { useResetSettings } from "@/lib/api/composables/settings.ts";
  import ConfirmDialog from "@/components/ui/tgui/ConfirmDialog.vue";
  import SettingsSectionWrapper from "@/components/settings/SettingsSectionWrapper.vue";
  import SettingsRow from "@/pages/components/settings/SettingsRow.vue";

  const { mutate: resetSettings } = useResetSettings();

  function handleResetSettings() {
    resetSettings(undefined, {
      onSuccess: () => toast.success("Settings reset to defaults"),
      onError: (err) =>
        toast.warning("Reset failed", {
          description: String(err),
        }),
    });
  }

  function handleClearHistory() {
    toast("Feature not implemented", {
      description: "History clearing will be available in a future update.",
    });
  }
</script>

<template>
  <SettingsSectionWrapper
    destructive
    title="Danger zone"
    description="These actions are irreversible. Please be certain before proceeding."
  >
    <SettingsRow
      label="Reset all settings"
      description="Restore every setting to its factory default value."
    >
      <ConfirmDialog
        title="Reset all settings?"
        description="All settings will be restored to their default values. This
				                cannot be undone."
        actionText="Reset"
        @confirm="handleResetSettings"
      >
        <Button variant="destructive" size="sm">Reset Settings</Button>
      </ConfirmDialog>
    </SettingsRow>

    <SettingsRow
      label="Clear all execution history"
      description="Permanently delete every execution history record in the database."
    >
      <ConfirmDialog
        title="Clear all execution history?"
        description="Every execution record will be permanently deleted. Running
                processes will not be affected but their history entries will be
                removed."
        actionText="Clear"
        @confirm="handleClearHistory"
      >
        <Button variant="destructive" size="sm">Clear History</Button>
      </ConfirmDialog>
    </SettingsRow>
  </SettingsSectionWrapper>
</template>

<script setup lang="ts">
  import { useDebounceFn } from "@vueuse/core";
  import { onMounted, ref, watch } from "vue";
  import { toast } from "vue-sonner";
  import SettingsRow from "@/components/settings/SettingsRow.vue";
  import SettingsSectionWrapper from "@/components/settings/SettingsSectionWrapper.vue";
  import { NumberField, NumberFieldInput } from "@/components/ui/number-field";
  import { Switch } from "@/components/ui/switch";
  import {
    useGetAllSettings,
    useSetSetting,
  } from "@/lib/api/composables/settings.ts";

  const { data: rawSettings } = useGetAllSettings();
  const { mutate: setSetting } = useSetSetting();

  const logBufferSize = ref(10_000);
  const autoScrollLogs = ref(true);
  const logRetentionDays = ref(30);

  onMounted(() => {
    watch(
      rawSettings,
      (s) => {
        if (!s) {
          return;
        }
        logBufferSize.value = Number(s.log_buffer_size) || 10_000;
        autoScrollLogs.value = s.auto_scroll_logs === "true";
        logRetentionDays.value = Number(s.log_retention_days) || 30;
      },
      { immediate: true }
    );
  });

  function save(key: string, value: string) {
    setSetting(
      { key, value },
      {
        onError: (err) =>
          toast({
            title: "Failed to save setting",
            description: String(err),
            variant: "destructive",
          }),
      }
    );
  }

  function saveBoolean(key: string, value: boolean) {
    save(key, value ? "true" : "false");
  }

  const saveNumberDebounced = useDebounceFn((key: string, value: number) => {
    if (!Number.isNaN(value) && value > 0) {
      save(key, String(value));
    }
  }, 600);
</script>

<template>
  <SettingsSectionWrapper
    title="Logs & Data"
    description="Configure logging behavior, output preferences and data-storage settings."
  >
    <SettingsRow
      label="Log Buffer Size"
      description="Number of log lines kept in memory per process."
    >
      <NumberField
        class="flex items-center"
        v-model="logBufferSize"
        :min="100"
        :max="100000"
        :step="1000"
        @update:model-value="(v) => saveNumberDebounced('log_buffer_size', v)"
      >
        <NumberFieldInput class="settings-input" />
        <span class="text-xs text-muted-foreground">Lines</span>
      </NumberField>
    </SettingsRow>

    <SettingsRow
      label="Auto-Scroll Logs"
      description="Automatically scroll to the latest log line as output arrives."
    >
      <Switch
        :checked="autoScrollLogs"
        @update:checked="(v: boolean) => { autoScrollLogs = v; saveBoolean('auto_scroll_logs', v) }"
      />
    </SettingsRow>

    <SettingsRow
      label="Log Retention"
      description="Automatically delete execution history older than this many days. Runs on app startup."
    >
      <NumberField
        class="flex items-center"
        v-model="logRetentionDays"
        :min="1"
        :max="365"
        @update:model-value="(v) => saveNumberDebounced('log_retention_days', v)"
      >
        <NumberFieldInput class="settings-input" />
        <span class="text-xs text-muted-foreground">days</span>
      </NumberField>
    </SettingsRow>
  </SettingsSectionWrapper>
</template>

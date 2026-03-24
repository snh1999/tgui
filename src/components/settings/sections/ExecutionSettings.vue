<script setup lang="ts">
  import { onMounted, ref, watch } from "vue";
  import { useDebounceFn } from "@vueuse/core";
  import { toast } from "vue-sonner";
  import SettingsRow from "@/pages/components/settings/SettingsRow.vue";
  import { Switch } from "@/components/ui/switch";
  import {
    useGetAllSettings,
    useSetSetting,
  } from "@/lib/api/composables/settings.ts";
  import SettingsSectionWrapper from "@/components/settings/SettingsSectionWrapper.vue";
  import ShellSelect from "@/components/forms/common/ShellSelect.vue";
  import { NumberField, NumberFieldInput } from "@/components/ui/number-field";

  const { data: rawSettings } = useGetAllSettings();
  const { mutate: setSetting } = useSetSetting();

  const defaultShell = ref("");
  const maxConcurrentProcesses = ref(20);
  const warnBeforeKill = ref(true);
  const killProcessTreeByDefault = ref(false);

  onMounted(() => {
    watch(
      rawSettings,
      (s) => {
        if (!s) return;
        defaultShell.value = s.default_shell ?? "";
        maxConcurrentProcesses.value = Number(s.max_concurrent_processes) || 20;
        warnBeforeKill.value = s.warn_before_kill === "true";
        killProcessTreeByDefault.value =
          s.kill_process_tree_by_default === "true";
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

  const saveStringDebounced = useDebounceFn((key: string, value: string) => {
    save(key, value);
  }, 600);

  const saveNumberDebounced = useDebounceFn((key: string, value: number) => {
    if (!Number.isNaN(value) && value > 0) save(key, String(value));
  }, 600);
</script>

<template>
  <SettingsSectionWrapper
    title="Execution"
    description="Configure how commands are executed and handled."
  >
    <SettingsRow
      label="Default Shell"
      description="Shell used when a command or group doesn't override it."
    >
      <!--			<Input-->
      <!--					:model-value="defaultShell"-->
      <!--					placeholder="/bin/bash"-->
      <!--					class="settings-input settings-input-mono"-->
      <!--					@update:model-value="(v) => { defaultShell = String(v); saveStringDebounced('default_shell', String(v)) }"-->
      <!--			/>-->

      <ShellSelect
        v-model="defaultShell"
        @update:model-value="(v: string) => saveStringDebounced('default_shell', v)"
      />
    </SettingsRow>

    <SettingsRow
      label="Max Concurrent Processes"
      description="Maximum number of processes that can run simultaneously."
    >
      <NumberField
        v-model="maxConcurrentProcesses"
        :min="1"
        :max="100"
        @update:model-value="(v) => saveNumberDebounced('max_concurrent_processes', v)"
      >
        <NumberFieldInput class="settings-input" />
      </NumberField>
    </SettingsRow>

    <SettingsRow
      label="Warn Before Kill"
      description="Show a confirmation dialog before stopping a running process."
    >
      <Switch
        :checked="warnBeforeKill"
        @update:checked="(v: boolean) => { warnBeforeKill = v; saveBoolean('warn_before_kill', v) }"
      />
    </SettingsRow>

    <SettingsRow
      label="Kill Process Tree by Default"
      description="When stopping a process, also kill its child processes."
    >
      <Switch
        :checked="killProcessTreeByDefault"
        @update:checked="(v: boolean) => { killProcessTreeByDefault = v; saveBoolean('kill_process_tree_by_default', v) }"
      />
    </SettingsRow>
  </SettingsSectionWrapper>
</template>

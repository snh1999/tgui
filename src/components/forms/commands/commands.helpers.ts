import { toTypedSchema } from "@vee-validate/zod";
import { split } from "shlex";
import { type FormContext, useForm } from "vee-validate";
import { computed, nextTick, ref, watch } from "vue";
import { z } from "zod";
import {
  argumentSchema,
  groupCommandFormSchema,
} from "@/components/forms/common/common.schema.ts";
import type { ICommand } from "@/lib/api/api.types.ts";
import {
  useCreateCommand,
  useUpdateCommand,
} from "@/lib/api/composables/commands.ts";
import { envVarsToArray, transformEnvVars } from "@/lib/helpers.ts";

export const commandFormSchema = groupCommandFormSchema.extend({
  command: z.string().min(1, "Command text can not be empty."),
  arguments: z.array(argumentSchema).nullable(),
  groupId: z.number().nullable().optional(),
});

export interface IUpsertCommandForm {
  command?: ICommand;
  isCreate?: boolean;
}

export function useCommandForm(
  props: IUpsertCommandForm,
  onSuccess: () => void
) {
  const { handleSubmit, resetForm, meta, values, setFieldValue } = useForm({
    validationSchema: toTypedSchema(commandFormSchema),
    initialValues: props.command
      ? {
          ...envVarsToArray(props.command),
          arguments: props.command.arguments ?? [],
        }
      : {
          name: "",
          command: "",
          id: 0,
          position: 0,
          envVars: [],
          isFavorite: false,
          arguments: [],
        },
  });

  const isDirty = computed(() => meta.value.dirty);
  const isValid = computed(() => meta.value.valid);

  const { mutate: createCommand, isPending: isCreatePending } =
    useCreateCommand();
  const { mutate: updateCommand, isPending: isUpdatePending } =
    useUpdateCommand();

  const isPending = computed(() =>
    props.command && !props.isCreate
      ? isUpdatePending.value
      : isCreatePending.value
  );

  const onSubmit = handleSubmit((rawData) => {
    const data = transformEnvVars(rawData);
    if (props.command && !props.isCreate) {
      updateCommand(
        { id: props.command.id, payload: data },
        { onSuccess: () => onSuccess() }
      );
    } else {
      createCommand(data, {
        onSuccess: () => {
          resetForm();
          onSuccess();
        },
      });
    }
  });

  return {
    resetForm,
    isPending,
    onSubmit,
    isDirty,
    isValid,
    values,
    setFieldValue,
  };
}

type TCommandFormInput = z.input<typeof commandFormSchema>;
/**
 * Provides a single "combined" input (e.g. `echo "hello world"`) that stays in sync with the separate `command` and `arguments` vee-validate fields.
 *
 * 1 — combined → fields:  triggered on blur, parsed with shlex.
 * 2 — fields → combined:  triggered when either field changes (user edit/reset)
 *
 * The `isSyncing` guard prevents the 2-watch from firing while 1 is writing, (could cause a update loop).
 */
export function useCommandLineSync(
  values: Partial<TCommandFormInput>,
  setFieldValue: FormContext<TCommandFormInput>["setFieldValue"]
) {
  const combined = ref<string>("");
  const isSyncing = ref(false);

  function buildCombined(cmd: string, args: string[]): string {
    return [cmd, ...(args ?? [])]
      .map((t) => t?.trim())
      .filter(Boolean)
      .join(" ");
  }

  combined.value = buildCombined(values.command ?? "", values.arguments ?? []);

  // fields → combined. (Deep-watch because arguments is an array)
  watch(
    () => [values.command, values.arguments] as [string, string[]],
    ([cmd, args]) => {
      if (isSyncing.value) {
        return;
      }
      combined.value = buildCombined(cmd ?? "", args ?? []);
    },
    { deep: true }
  );

  // combined → fields
  function onCombinedChange() {
    const input = combined.value.trim();

    if (!input) {
      setFieldValue("command", "");
      setFieldValue("arguments", []);
      return;
    }

    let tokens: string[];
    try {
      tokens = split(input);
    } catch {
      tokens = input.split(/\s+/);
    }

    const [cmd = "", ...rest] = tokens;

    isSyncing.value = true;
    setFieldValue("command", cmd);
    setFieldValue("arguments", rest);

    combined.value = buildCombined(cmd, rest);

    nextTick(() => {
      isSyncing.value = false;
    });
  }

  return { combined, onCombinedChange };
}

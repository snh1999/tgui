import { toTypedSchema } from "@vee-validate/zod";
import { useForm } from "vee-validate";
import { computed } from "vue";
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
}

export function useCommandForm(
  props: IUpsertCommandForm,
  onSuccess: () => void
) {
  const { handleSubmit, resetForm, meta } = useForm({
    validationSchema: toTypedSchema(commandFormSchema),
    initialValues: props.command
      ? {
          ...envVarsToArray(props.command),
          arguments: props.command.arguments ?? [],
        }
      : {
          name: "",
          command: "",
          description: "",
          id: 0,
          workingDirectory: "",
          position: 0,
          envVars: [],
          isFavorite: false,
          shell: "",
          arguments: [""],
        },
  });

  const isDirty = computed(() => meta.value.dirty);
  const isValid = computed(() => meta.value.valid);

  const { mutate: createCommand, isPending: isCreatePending } =
    useCreateCommand();
  const { mutate: updateCommand, isPending: isUpdatePending } =
    useUpdateCommand();

  const isPending = computed(() =>
    props.command ? isUpdatePending.value : isCreatePending.value
  );

  const onSubmit = handleSubmit((rawData) => {
    const data = transformEnvVars(rawData);
    if (props.command) {
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

  return { resetForm, isPending, onSubmit, isDirty, isValid };
}

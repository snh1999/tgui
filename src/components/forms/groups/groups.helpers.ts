import { toTypedSchema } from "@vee-validate/zod";
import { useForm } from "vee-validate";
import { computed } from "vue";
import { z } from "zod";
import { groupCommandFormSchema } from "@/components/forms/common/common.schema.ts";
import type { IGroup } from "@/lib/api/api.types.ts";
import {
  useCreateGroup,
  useUpdateGroup,
} from "@/lib/api/composables/groups.ts";
import { envVarsToArray, transformEnvVars } from "@/lib/helpers.ts";

export const groupFormSchema = groupCommandFormSchema.extend({
  parentGroupId: z.number().nullable().optional(),
  icon: z.string().optional().nullable(),
});

export interface IUpsertGroupForm {
  group?: IGroup;
  isCreate?: boolean;
}

export function useGroupForm(props: IUpsertGroupForm, onSuccess: () => void) {
  const { handleSubmit, resetForm, meta } = useForm({
    validationSchema: toTypedSchema(groupFormSchema),
    initialValues: props.group
      ? envVarsToArray(props.group)
      : {
          name: "",
          description: "",
          id: 0,
          workingDirectory: "",
          position: 0,
          envVars: [],
          isFavorite: false,
          shell: "",
        },
  });

  const isValid = computed(() => meta.value.valid);
  const isDirty = computed(() => meta.value.dirty);

  const { mutate: createGroup, isPending: isCreatePending } = useCreateGroup();
  const { mutate: updateGroup, isPending: isUpdatePending } = useUpdateGroup();

  const isPending = computed(() =>
    props.group ? isUpdatePending.value : isCreatePending.value
  );

  const onSubmit = handleSubmit((rawData) => {
    const data = transformEnvVars(rawData);
    if (props.group && !props.isCreate) {
      updateGroup(
        { id: props.group.id, payload: data },
        { onSuccess: () => onSuccess() }
      );
    } else {
      createGroup(data, {
        onSuccess: () => {
          resetForm();
          onSuccess();
        },
      });
    }
  });

  return { resetForm, isPending, onSubmit, isDirty, isValid };
}

import { z } from "zod";
import { ICategory } from "@/lib/api/api.types.ts";
import { useForm } from "vee-validate";
import { toTypedSchema } from "@vee-validate/zod";
import { computed } from "vue";
import {
  useCreateCategory,
  useUpdateCategory,
} from "@/lib/api/composables/categories.ts";

export const categoryFormSchema = z.object({
  id: z.number().default(0),
  name: z
    .string()
    .min(3, "Name must be at least 3 characters.")
    .max(32, "Name must be less than 32 characters."),
  icon: z.string().optional().nullable(),
  color: z.string().optional().nullable(),
});

export interface IUpsertCategoryForm {
  category: ICategory;
}

export function useCategoryForm(
  props: IUpsertCategoryForm,
  onSuccess: () => void
) {
  const { handleSubmit, resetForm, meta } = useForm({
    validationSchema: toTypedSchema(categoryFormSchema),
    initialValues: props.category
      ? props.category
      : {
          name: "",
          icon: "",
          color: "",
        },
  });

  const isValid = computed(() => meta.value.valid);
  const isDirty = computed(() => meta.value.pending);

  const { mutate: createCategory, isPending: isCreatePending } =
    useCreateCategory();
  const { mutate: updateCategory, isPending: isUpdatePending } =
    useUpdateCategory();

  const isPending = computed(() =>
    props.category ? isUpdatePending.value : isCreatePending.value
  );

  const onSubmit = handleSubmit((data) => {
    if (props.category) {
      updateCategory(
        { id: props.category.id, payload: data },
        { onSuccess: () => onSuccess() }
      );
    } else {
      createCategory(data, {
        onSuccess: () => {
          resetForm();
          onSuccess();
        },
      });
    }
  });

  return { isDirty, isValid, isPending, resetForm, onSubmit };
}

import { z } from "zod";

export const envVarEntrySchema = z.object({
  key: z
    .string()
    .min(1, "Key cannot be empty")
    .regex(/^[a-zA-Z_][a-zA-Z0-9_]*$/, "Invalid env var name"),
  value: z.string(), // Allow empty values for unset
});

const commonFormSchema = z.object({
  name: z
    .string()
    .min(3, "Command name must be at least 3 characters.")
    .max(32, "Command name must be less than 32 characters."),
  description: z.string().optional().nullable(),
  position: z.number().default(0),
  id: z.number().default(0),
  isFavorite: z.boolean().default(false),
});

export const groupCommandFormSchema = commonFormSchema.extend({
  // using array because form input processing is easier that way
  envVars: z.array(envVarEntrySchema).default([]),
  categoryId: z.number().nullable().optional(),
  workingDirectory: z.string().optional().nullable(),
  shell: z.string().optional().nullable(),
});

export const argumentSchema = z.string().refine(
  (val) => {
    if (val === "") {
      return true;
    }
    const hasUnbalancedQuotes = (val.match(/"/g) || []).length % 2 !== 0;
    return !hasUnbalancedQuotes;
  },
  {
    message: "Unbalanced quotes in argument",
  }
);

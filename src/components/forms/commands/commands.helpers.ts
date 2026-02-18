import { z } from "zod";

const argumentSchema = z.string().refine(
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

const envVarEntrySchema = z.object({
  key: z
    .string()
    .min(1, "Key cannot be empty")
    .regex(/^[a-zA-Z_][a-zA-Z0-9_]*$/, "Invalid env var name"),
  value: z.string(), // Allow empty values for unset
});

export const commandFormSchema = z.object({
  name: z
    .string()
    .min(3, "Command name must be at least 3 characters.")
    .max(32, "Command name must be less than 32 characters."),
  command: z.string().min(1, "Command text can not be empty."),
  arguments: z.array(argumentSchema).default([""]),
  description: z.string().optional(),
  groupId: z.number().nullable().optional(),
  position: z.number().default(0),
  id: z.number().default(0),
  workingDirectory: z.string().optional(),
  // using array because form input processing is easier that way
  envVars: z.array(envVarEntrySchema).default([]),
  shell: z.string().optional(),
  categoryId: z.number().nullable().optional(),
  isFavorite: z.boolean().default(false),
});

import { z } from "zod";

export interface ICommand {
  id: number;
  name: string;
  description?: string;
  command: string;
  arguments?: string[];
  env_vars?: Map<string, string>;
  group_id?: number;
  category_id?: number;
  position: number;
  working_directory?: string;
  shell?: string;
  is_favorite?: boolean;
  created_at?: string;
  updated_at?: string;
}

const argumentSchema = z.string().refine(
  (val) => {
    if (val === "") return true;

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
  group_id: z.number().optional(),
  position: z.number().default(0),
  id: z.number().default(0),
  working_directory: z.string().optional(),
  // using array because form input processing is easier that way
  env_vars: z.array(envVarEntrySchema).default([]),
  shell: z.string().optional(),
  category_id: z.number().optional(),
  is_favorite: z.boolean().default(false),
});

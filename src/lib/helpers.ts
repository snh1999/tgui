export function transformEnvVars<
  T extends {
    env_vars: { key: string; value: string }[];
  },
>(values: T) {
  return {
    ...values,
    env_vars: Object.fromEntries(values.env_vars.map((e) => [e.key, e.value])),
  };
}

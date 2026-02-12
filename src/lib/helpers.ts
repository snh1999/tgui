export function transformEnvVars<
  T extends {
    env_vars: { key: string; value: string }[];
  },
>(values: T) {
  return {
    ...values,
    env_vars: new Map(
      values.env_vars
        .map((e) => ({ key: e.key.trim(), value: e.value }))
        .filter((e) => e.key.length > 0)
        .map((e) => [e.key, e.value])
    ),
  };
}

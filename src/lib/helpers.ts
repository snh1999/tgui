export function transformEnvVars<
  T extends {
    envVars: { key: string; value: string }[];
  },
>(values: T) {
  return {
    ...values,
    envVars: new Map(
      values.envVars
        .map((e) => ({ key: e.key.trim(), value: e.value }))
        .filter((e) => e.key.length > 0)
        .map((e) => [e.key, e.value])
    ),
  };
}

export function envVarsToArray<
  T extends {
    envVars?: Map<string, string>;
  },
>(values: T) {
  return {
    ...values,
    envVars: values.envVars
      ? Object.keys(values.envVars).map((key) => ({
          key,
          value: values.envVars?.get(key) ?? "",
        }))
      : undefined,
  };
}

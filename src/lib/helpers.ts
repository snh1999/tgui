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
    envVars?: Map<string, string> | Record<string, string> | null;
  },
>(values: T) {
  const toEntries = () => {
    if (!values.envVars) {
      return null;
    }
    if (values.envVars instanceof Map) {
      return Array.from(values.envVars.entries());
    }
    return Object.entries(values.envVars);
  };

  return {
    ...values,
    envVars: toEntries()?.map(([key, value]) => ({ key, value })),
  };
  // return {
  //   ...values,
  //   envVars: values.envVars
  //     ? Array.from(values.envVars.entries()).map(([key, value]) => ({
  //         key,
  //         value,
  //       }))
  //     : undefined,
  // };
}

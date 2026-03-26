import { type ComputedRef, computed } from "vue";
import type { ICommandGroupFilter } from "@/lib/api/api.types.ts";
import { useGetCommands } from "@/lib/api/composables/commands.ts";
import { useGetGroups } from "@/lib/api/composables/groups.ts";

export function useGetGroupCommand(filters: ComputedRef<ICommandGroupFilter>) {
  const {
    data: commands,
    isPending: commandsLoading,
    isError: commandsError,
    error: commandsErr,
    refetch: refetchCommands,
  } = useGetCommands(filters);

  const {
    data: groups,
    isPending: groupsLoading,
    isError: groupsError,
    error: groupsErr,
    refetch: refetchGroups,
  } = useGetGroups(filters);

  const isValuesLoading = computed(
    () => commandsLoading.value || groupsLoading.value
  );
  const isError = computed(() => commandsError.value || groupsError.value);
  const error = computed(() => commandsErr.value || groupsErr.value);
  const refetch = computed(() => refetchCommands || refetchGroups);

  return {
    groups,
    commands,
    isValuesLoading,
    isError,
    error,
    refetch,
  };
}

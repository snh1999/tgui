<script setup lang="ts">
import { ref } from "vue";
import { AddIcon } from "@/assets/Icons";
import OpenDialog from "@/components/ui/tgui/OpenDialog.vue";
import { Button } from "@/components/ui/button";
import CreateCommandForm from "@/components/forms/commands/CreateCommandForm.vue";

const newCommandOpen = ref(false);
</script>

<template>
    <header class="header">
        <Button @click="newCommandOpen = true" class="btn-primary gap-2">
            <AddIcon />
            New Command
        </Button>

        <OpenDialog v-model:open="newCommandOpen" title="Create New Command">
            <CreateCommandForm />

            <template #footer>
                <Button variant="outline" @click="newCommandOpen = false"
                    >Cancel</Button
                >
                <Button @click="newCommandOpen = false">Create</Button>
            </template>
        </OpenDialog>
    </header>
</template>

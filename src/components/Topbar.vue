<script setup lang="ts">
  import { AddIcon, SearchIcon, SettingsIcon } from "@/assets/Icons";
</script>

<template>
  <header class="header">
    <div class="search-bar">
      <span class="search-icon">
        <SearchIcon />
      </span>

      <input type="text" placeholder="Search commands...">
    </div>
    <div class="header-actions">
      <button class="table-btn btn-settings flex gap-1.5 items-center">
        <SettingsIcon />
        Settings
      </button>
      <button class="table-btn btn-primary flex items-center gap-1.5">
        <AddIcon />
        New command
      </button>
    </div>
  </header>
</template>

<style scoped>
  .header {
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    padding: var(--space-md) var(--space-lg);
    flex-shrink: 0;
  }

  .search-bar {
    flex: 1;
    position: relative;
  }

  .search-bar input {
    width: 100%;
    padding: 10px 40px;
    border: 1px solid #e5e5e5;
    border-radius: 8px;
    font-size: 14px;
  }

  .search-icon {
    position: absolute;
    left: 12px;
    top: 50%;
    transform: translateY(-50%);
    color: #999;
  }

  .header-actions {
    display: flex;
    gap: 10px;
  }

  .btn-settings {
    background: white;
    border: 1px solid #e5e5e5;
  }

  @container root (max-width: 600px) {
    .search-bar {
      max-width: 200px;
    }

    .btn-settings span {
      display: none; /* Hide "Settings" text, keep icon */
    }
  }

  @container root (max-width: 450px) {
    .search-bar input::placeholder {
      color: transparent; /* Hide placeholder to save space */
    }

    .header-actions button {
      padding: var(--space-sm);
    }
  }
</style>

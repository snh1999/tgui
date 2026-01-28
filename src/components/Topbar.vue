<script setup lang="ts">
import {AddIcon, SearchIcon, SettingsIcon} from '@/assets/Icons'
</script>

<template>
  <header class="header">
    <div class="search-bar">
      <span class="search-icon">
        <SearchIcon/>
      </span>

      <input type="text" placeholder="Search commands...">
    </div>
    <div class="header-actions">
      <button class="table-btn btn-settings flex gap-1.5 items-center">
        <SettingsIcon/>
        Settings
      </button>
      <button class="table-btn btn-primary flex items-center gap-1.5">
        <AddIcon/>
        New command
      </button>
    </div>
  </header>
</template>

<style scoped>
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
</style>

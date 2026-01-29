<script setup lang="ts">
  import {
    AddIcon,
    DashboardIcon,
    DirectoryIcon,
    PlayCircleIcon,
  } from "@/assets/Icons.ts";
  import logo from "@/assets/logo.svg";
  import HistoryIcon from "~icons/iconamoon/history-bold";
</script>

<template>
  <aside class="sidebar flex flex-col bg-white p-5">
    <div class="flex items-center justify-center gap-3 text-xl mb-5">
      <img class="h-8 w-8" alt="logo" :src="logo">
      <div class="logo-text">TGUI</div>
    </div>

    <div class="nav-section">
      <div class="nav-label mb-3">Main</div>
      <div class="nav-item active">
        <DashboardIcon />

        <span>Dashboard</span>
      </div>
      <div class="nav-item">
        <PlayCircleIcon />
        <span>Active Processes</span>
        <span class="badge">3</span>
      </div>
      <div class="nav-item">
        <HistoryIcon />
        <span>History</span>
      </div>
    </div>

    <div class="nav-section">
      <div class="flex justify-between items-center mb-2">
        <div class="nav-label">Groups</div>
        <div
          class="flex rounded-md justify-center btn-primary px-2.5 py-2 gap-1"
        >
          <AddIcon class="text-xs text-white " />
          <span class="text-white text-xs">New</span>
        </div>
      </div>

      <div class="nav-item">
        <DirectoryIcon />
        <span>Frontend Build</span>
      </div>
      <div class="nav-item">
        <DirectoryIcon />
        <span>Docker Services</span>
      </div>
    </div>
  </aside>
</template>

<style scoped>
  .sidebar {
    width: var(--sidebar-width);
    border-right: 1px solid var(--color-border);
    background: var(--color-surface);
    transition: width var(--transition-base);
    overflow: hidden;
    flex-shrink: 0;
  }

  .nav-item.active {
    background: var(--color-bg);
    color: var(--color-primary);
  }

  .logo-text {
    font-size: 18px;
    font-weight: 600;
    letter-spacing: 0.5px;
  }

  .nav-section {
    margin-bottom: 30px;
  }

  .nav-label {
    font-size: 11px;
    text-transform: uppercase;
    color: #999;
    font-weight: 600;
    letter-spacing: 0.5px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 5px;
    border-radius: 6px;
    cursor: pointer;
    margin-bottom: 4px;
    color: var(--color-text-muted);
    font-weight: 600;
    transition: background 0.2s;
  }

  .nav-item:hover {
    opacity: 0.8;
  }

  .nav-item .badge {
    margin-left: auto;
    background: #f0f0f0;
    padding: 2px 8px;
    border-radius: 10px;
    font-size: 12px;
  }

  @container root (max-width: 500px) {
    .sidebar {
      width: var(--sidebar-collapsed-width);
      padding: var(--space-sm);
    }

    .logo-text,
    .nav-item span,
    .nav-label,
    .btn-primary span {
      display: none;
    }

    .nav-item {
      justify-content: center;
      padding: var(--space-sm);
    }

    .badge {
      position: absolute;
      top: 0;
      right: 0;
      font-size: 10px;
      padding: 2px 4px;
    }
  }
</style>

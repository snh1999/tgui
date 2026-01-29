<script setup lang="ts">
  import {
    DirectoryIcon,
    FilledStarIcon,
    MenuDotsIcon,
    PlayIcon,
    RestartIcon,
    StarIcon,
  } from "@/assets/Icons.ts";

  defineProps<{
    name: string;
    description?: string;
    activityTime: string;
    pid?: number; //TODO consider changing to message?
    status: "running" | "stopped" | "error";
    working_directory: string;
    isFavorite?: boolean;
    group_name?: string;
    category_name?: string;
  }>();
</script>

<template>
  <tr>
    <td>
      <!--  TODO hover text is command description    -->
      <div class="command-name">
        <div class="command-icon">
          <FilledStarIcon v-if="isFavorite" />
          <StarIcon v-else />
        </div>

        <div class="command-details">
          <div class="name">{{ name }}</div>
          <div class="desc">{{ description }}</div>
        </div>
      </div>
    </td>
    <td>
      <div class="status" :class="status">
        <span class="status-dot" />
        {{ status[0].toUpperCase() + status.slice(1) }}
      </div>
      <div class="desc pt-2 flex gap-1">
        <DirectoryIcon />
        {{ working_directory }}
      </div>
    </td>
    <td>
      <div class="execution-time">
        {{ activityTime }}
        <!-- TODO add additonal is running check-->
        <span v-if="pid" class="pid">PID: {{ pid }}</span>
      </div>
    </td>
    <td>
      <button class="table-btn">
        <PlayIcon />
      </button>
      <button class="table-btn">
        <RestartIcon />
      </button>
      <button class="table-btn">
        <MenuDotsIcon />
      </button>
    </td>
    <!--    <td>-->
    <!--      <span class="tag">{{ group_name }}</span>-->
    <!--      <span class="tag">{{ category_name }}</span>-->
    <!--    </td>-->
  </tr>
</template>

<style scoped>
  td {
    padding: 16px 20px;
    border-bottom: 1px solid #f3f4f6;
  }

  .command-name {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .command-icon {
    width: 32px;
    height: 32px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 18px;
  }

  .command-details .name {
    font-weight: 500;
    margin-bottom: 4px;
  }

  .table-btn {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 12px;
    color: #999;
    padding: 6px;
  }

  .table-btn:hover {
    color: #777;
  }

  .desc {
    font-size: 12px;
    color: #999;
  }

  .tag {
    display: inline-block;
    padding: 4px 10px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    margin-right: 6px;
  }

  .status {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .status.running .status-dot {
    background: #10b981;
  }

  .status.stopped .status-dot {
    background: #6b7280;
  }

  .status.error .status-dot {
    background: #ef4444;
  }

  .execution-time {
    color: #666;
    font-size: 13px;
  }

  .pid {
    display: block;
    font-size: 11px;
    color: #999;
    margin-top: 2px;
  }

  @container root (max-width: 700px) {
    .desc {
      display: none;
    }

    td {
      padding: var(--space-sm) var(--space-md);
      white-space: nowrap;
    }
  }

  @container root (max-width: 500px) {
    .status {
      font-size: 11px;
    }

    .status-text {
      display: none;
    }

    .status-dot {
      margin: 0;
    }
  }
</style>

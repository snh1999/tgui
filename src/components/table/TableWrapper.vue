<script setup lang="ts">
import TableRow from "@/components/table/TableRow.vue";
</script>

<template>
  <section class="list">
    <div class="list-header">
      <div class="list-title">
        Command Registry
        <span class="list-badge">8 active</span>
      </div>
      <div class="list-actions">
        <button class="table-btn">☰</button>
        <button class="table-btn">≡</button>
      </div>
    </div>

    <div class="table-scroll-container">
      <table>
        <thead>
        <tr>
          <th>COMMAND NAME</th>
          <th>STATUS</th>
          <th>LAST EXECUTION</th>
          <th>ACTIONS</th>
        </tr>
        </thead>
        <tbody>
        <TableRow
            description="npm run dev"
            name="Dev server"
            activityTime="2 minutes ago"
            :pid="45092"
            color="#9333ea;"
            status="running"
            working_directory="~/dev"
        />

        <TableRow
            description="docker-compose up"
            name="Postgresql"
            activityTime="Yesterday, 18:45"
            color="#2563eb;"
            status="stopped"
            working_directory="/os/home"
        />

        <TableRow
            description="npm run test:watch"
            name="Unit Tests"
            activityTime="5 seconds ago"
            color="#ef4444;"
            status="error"
            working_directory="~/dev/documents/process"
        />

        </tbody>
      </table>
      <div class="table-footer">
        <div>Showing 3 of 12 commands</div>
        <div class="pagination">
          <button disabled>Previous</button>
          <button>Next</button>
        </div>
      </div>
    </div>
  </section>

</template>

<style scoped>
.list {
  padding: 0;
  background: var(--color-surface);
  border-radius: 12px;
  border: 1px solid var(--color-border);
}

.list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.list-title {
  font-size: 20px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 10px;
}

.list-badge {
  background: #f0f0f0;
  padding: 4px 10px;
  border-radius: 12px;
  font-size: 13px;
  color: #666;
}

.list-actions {
  display: flex;
  gap: 8px;
}

.table-scroll-container {
  overflow-x: auto;
  overflow-y: hidden;
  -webkit-overflow-scrolling: touch;
  /* Show scrollbar on hover only for cleaner look */
  scrollbar-width: thin;
}

.table-scroll-container::-webkit-scrollbar {
  height: 6px;
}

.table-scroll-container::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 3px;
}

table {
  min-width: 600px;
  width: 100%;
}

thead {
  background: var(--color-bg);
}

table {
  width: 100%;
  border-collapse: collapse;
}

thead {
  background: #f9fafb;
  border-bottom: 1px solid #e5e5e5;
}

th {
  text-align: left;
  padding: 12px 20px;
  font-size: 11px;
  text-transform: uppercase;
  color: #666;
  font-weight: 600;
  letter-spacing: 0.5px;
}

.table-footer {
  padding: 16px 20px;
  background: #f9fafb;
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 13px;
  color: #666;
}

.pagination {
  display: flex;
  gap: 8px;
}

.pagination button {
  padding: 6px 12px;
  border: 1px solid #e5e5e5;
  background: white;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
}

.pagination button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>

<script lang="ts">
  import SchedulePanel from "../schedule/SchedulePanel.svelte";
  import LogPanel from "./LogPanel.svelte";
  import SyncPanel from "./SyncPanel.svelte";

</script>

<div class="dashboard-stats">
    <div class="stats-layout">
			<SyncPanel />
      <SchedulePanel />
			<LogPanel />
    </div>
</div>

<style>
  .dashboard-stats {
    padding: 0.5rem;
    overflow-x: auto;
  }

  .stats-layout {
    display: flex;
    gap: 0.75rem;
    width: 100%;
  }
</style>
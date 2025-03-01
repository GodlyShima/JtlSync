<script lang="ts">
  import { faTerminal } from "@fortawesome/free-solid-svg-icons";
  import PanelHeader from "./PanelHeader.svelte";
</script>

<div class="stat-panel">
  <PanelHeader icon={faTerminal} title="Logs" />
  <div class="system-grid">
  </div>
</div>

<style>
  .stat-panel {
    flex: 0.8;
    min-width: 125px;
    background-color: var(--mantle);
    border-radius: 6px;
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
  }

  .system-grid {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    flex: 1;
  }
</style>
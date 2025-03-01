<script lang="ts">
  import { faDatabase } from "@fortawesome/free-solid-svg-icons";
  import PanelHeader from "./PanelHeader.svelte";
  import StatItem from "./StatItem.svelte";
</script>

<div class="stat-panel">
  <PanelHeader icon={faDatabase} title="Synchronisierungsdetails" />
  <div class="system-grid">
    <StatItem color="#a7c957" label="Abgeschlossen" value={"0"} />
    <StatItem color="#fca311" label="Übersprungen" value={"0"} />
    <StatItem color="#e63946" label="Fehlgeschlagen" value={"0"} />
    <StatItem label="Nächste Synchronisation startet" value={"02.03.2025 01:00 Uhr"} />
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
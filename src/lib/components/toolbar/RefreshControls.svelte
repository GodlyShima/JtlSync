<script lang="ts">
  import { REFRESH_RATE_OPTIONS } from "$lib/constants";
  import { settingsStore } from "$lib/stores/settings";
  import type { ToolConfig } from "$lib/types";
  import { faPause, faPlay } from "@fortawesome/free-solid-svg-icons";
  import Fa from "svelte-fa";
  export let refreshRate: number;
  export let isFrozen: boolean;

  function updateBehaviorConfig(key: keyof ToolConfig["behavior"], value: any) {
    settingsStore.updateConfig({
      behavior: {
        ...$settingsStore.behavior,
        [key]: value,
      },
    });
  }
</script>

<div class="refresh-controls">
  <select
    class="select-input"
    bind:value={refreshRate}
    on:change={() => updateBehaviorConfig("refreshRate", refreshRate)}
    disabled={isFrozen}
  >
    {#each REFRESH_RATE_OPTIONS as option}
      <option value={option.value}>{option.label}</option>
    {/each}
  </select>
  <button
    class="btn-action"
    class:frozen={isFrozen}
    on:click={() => (isFrozen = !isFrozen)}
    title={isFrozen ? "Resume Updates" : "Pause Updates"}
  >
    {#if isFrozen}
      <Fa icon={faPlay} color="var(--red)" />
    {:else}
      <Fa icon={faPause} color="var(--subtext0)" />
    {/if}
  </button>
</div>

<style>
  .refresh-controls {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .refresh-controls :global(svg) {
    font-size: 14px;
    color: var(--subtext0);
  }

  .btn-action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: none;
    background: var(--surface0);
    border: 1px solid var(--surface1);
    color: var(--text);
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-action:hover {
    background: var(--surface1);
  }

  .btn-action.frozen {
    background: var(--yellow);
  }

  .select-input {
    height: 28px;
    padding: 0 8px;
    border: 1px solid var(--surface1);
    border-radius: 6px;
    background: var(--surface0);
    color: var(--text);
    font-size: 13px;
    cursor: pointer;
    appearance: none;
    padding-right: 24px;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%23cdd6f4' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 8px center;
  }

  .select-input:hover {
    background-color: var(--surface1);
  }

  .select-input:focus {
    outline: none;
    border-color: var(--blue);
  }

  .select-input:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }
</style>
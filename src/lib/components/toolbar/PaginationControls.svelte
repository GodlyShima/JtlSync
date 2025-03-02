<script lang="ts">
  import { ITEMS_PER_PAGE_OPTIONS } from "$lib/constants";
  import { settingsStore } from "$lib/stores/settings";
  import type { ToolConfig } from "$lib/types";

  export let itemsPerPage: number;
  export let currentPage: number;
  export let totalPages: number;
  export let totalResults: number;

  function changePage(page: number) {
    if (page >= 1 && page <= totalPages) {
      currentPage = page;
    }
  }

  function updateBehaviorConfig(key: keyof ToolConfig["behavior"], value: any) {
    settingsStore.updateConfig({
      behavior: {
        ...$settingsStore.behavior,
        [key]: value,
      },
    });
  }
</script>

<div class="pagination-controls">
  <select
    class="select-input"
    bind:value={itemsPerPage}
    on:change={() => updateBehaviorConfig("itemsPerPage", itemsPerPage)}
    aria-label="Items per page"
  >
    {#each ITEMS_PER_PAGE_OPTIONS as option}
      <option value={option}>{option} per page</option>
    {/each}
  </select>

  <div class="pagination">
    <button
      class="btn-page"
      disabled={currentPage === 1}
      on:click={() => changePage(1)}
    >
      ««
    </button>
    <button
      class="btn-page"
      disabled={currentPage === 1}
      on:click={() => changePage(currentPage - 1)}
    >
      «
    </button>
    <div class="page-info">
      <span>Seite {currentPage} von {totalPages}</span>
      <span class="results-info">({totalResults} Bestellungen)</span>
    </div>
    <button
      class="btn-page"
      disabled={currentPage === totalPages}
      on:click={() => changePage(currentPage + 1)}
    >
      »
    </button>
    <button
      class="btn-page"
      disabled={currentPage === totalPages}
      on:click={() => changePage(totalPages)}
    >
      »»
    </button>
  </div>
</div>

<style>
  .pagination-controls {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .select-input {
    padding: 6px 12px;
    font-size: 12px;
    color: var(--text);
    background: var(--surface0);
    border: 1px solid var(--surface1);
    border-radius: 6px;
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

  .pagination {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .btn-page {
    padding: 6px 10px;
    font-size: 12px;
    color: var(--text);
    background: var(--surface0);
    border: 1px solid var(--surface1);
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-page:hover:not(:disabled) {
    background: var(--surface1);
  }

  .btn-page:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .page-info {
    font-size: 12px;
    color: var(--subtext0);
    display: flex;
    flex-direction: column;
    align-items: center;
    flex-shrink: 0;
  }

  .page-info span {
    display: block;
  }

  .results-info {
    color: var(--overlay0);
  }
</style>
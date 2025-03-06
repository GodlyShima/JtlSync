<script lang="ts">
  import type { Column } from "$lib/types";
  import { faCog, faSort, faSortDown, faSortUp } from "@fortawesome/free-solid-svg-icons";
  import { createEventDispatcher } from "svelte";
  import Fa from "svelte-fa";
  import { slide } from "svelte/transition";

  export let columns: Column[];
  
  // Sorting state
  let sortColumn: string | null = null;
  let sortDirection: 'asc' | 'desc' = 'asc';
  let showColumnSettings = false;
  
  // Create event dispatcher
  const dispatch = createEventDispatcher();
  
  // Handle column click for sorting
  function handleColumnClick(columnId: string) {
    if (sortColumn === columnId) {
      // Toggle direction if clicking the same column
      sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
    } else {
      // Set new column and default to ascending
      sortColumn = columnId;
      sortDirection = 'asc';
    }
    
    // Dispatch the sort event to parent
    dispatch('sort', { column: sortColumn, direction: sortDirection });
  }
  
  // Toggle column settings panel
  function toggleColumnSettings() {
    showColumnSettings = !showColumnSettings;
  }
  
  // Toggle column visibility
  function toggleColumnVisibility(columnId: string) {
    // We need to find the column and toggle its visibility
    const index = columns.findIndex(col => col.id === columnId);
    if (index >= 0 && !columns[index].required) {
      columns[index].visible = !columns[index].visible;
      // Notify parent
      dispatch('columnsChange', columns);
    }
  }
</script>

<thead>
  <tr>
    {#each columns.filter((col) => col.visible) as column (column.id)}
      <th 
        class="sortable"
        data-column={column.id}
        on:click={() => handleColumnClick(column.id)}
      >
        <div class="th-content">
          {column.label}
          <span class="sort-indicator" class:active={sortColumn === column.id}>
            {#if sortColumn === column.id}
              <Fa icon={sortDirection === 'asc' ? faSortUp : faSortDown} />
            {:else}
              <Fa icon={faSort} />
            {/if}
          </span>
        </div>
      </th>
    {/each}
    <th class="actions-header">
      <button class="column-settings-btn" on:click={toggleColumnSettings} title="Spalten-Einstellungen">
        <Fa icon={faCog} />
      </button>
    </th>
  </tr>
</thead>

{#if showColumnSettings}
  <div class="column-settings-panel" transition:slide={{ duration: 150 }}>
    <div class="panel-header">
      <h4>Spalten anzeigen/verstecken</h4>
      <button class="close-btn" on:click={toggleColumnSettings}>×</button>
    </div>
    <div class="column-toggles">
      {#each columns as column}
        <label class="column-toggle">
          <input 
            type="checkbox" 
            checked={column.visible} 
            on:change={() => toggleColumnVisibility(column.id)}
            disabled={column.required}
          />
          <span class="toggle-label">{column.label}</span>
          {#if column.required}
            <span class="required-badge">Erforderlich</span>
          {/if}
        </label>
      {/each}
    </div>
  </div>
{/if}

<style>
  th {
    position: sticky;
    top: 0;
    background: var(--surface0);
    text-align: left;
    padding: 0.6rem 0.8rem;
    font-weight: 500;
    color: var(--subtext0);
    border-bottom: 1px solid var(--surface1);
    transition: background-color var(--transition-fast);
    z-index: 2;
    font-size: 0.8rem;
  }
  
  th:hover {
    background-color: var(--surface1);
  }

  .actions-header {
    width: 90px;
    min-width: 90px;
    max-width: 90px;
    text-align: center;
  }

  .sortable {
    cursor: pointer;
    user-select: none;
  }

  .th-content {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .sort-indicator {
    color: var(--overlay0);
    font-size: 0.75rem;
    opacity: 0.5;
    transition: all var(--transition-fast);
  }

  .sort-indicator.active {
    color: var(--blue);
    opacity: 1;
  }

  .sortable:hover .sort-indicator {
    opacity: 1;
  }
  
  .column-settings-btn {
    background: none;
    border: none;
    color: var(--subtext0);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    margin: 0 auto;
    border-radius: var(--border-radius-sm);
    transition: all var(--transition-fast);
  }
  
  .column-settings-btn:hover {
    background-color: var(--surface1);
    color: var(--text);
  }
  
  .column-settings-panel {
    position: absolute;
    top: 45px;
    right: 20px;
    background-color: var(--mantle);
    border-radius: var(--border-radius-md);
    box-shadow: var(--shadow-lg);
    width: 250px;
    z-index: 10;
    border: 1px solid var(--surface0);
  }
  
  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem;
    border-bottom: 1px solid var(--surface0);
  }
  
  .panel-header h4 {
    margin: 0;
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--text);
  }
  
  .close-btn {
    background: none;
    border: none;
    color: var(--text);
    font-size: 1.2rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    padding: 0;
    border-radius: var(--border-radius-sm);
  }
  
  .close-btn:hover {
    background-color: var(--surface1);
  }
  
  .column-toggles {
    max-height: 300px;
    overflow-y: auto;
    padding: 0.5rem;
  }
  
  .column-toggle {
    display: flex;
    align-items: center;
    padding: 0.5rem;
    border-radius: var(--border-radius-sm);
    cursor: pointer;
    transition: background-color var(--transition-fast);
  }
  
  .column-toggle:hover {
    background-color: var(--surface0);
  }
  
  .toggle-label {
    margin-left: 0.5rem;
    font-size: 0.8rem;
  }
  
  .required-badge {
    font-size: 0.65rem;
    padding: 0.1rem 0.3rem;
    background-color: var(--surface1);
    color: var(--text);
    border-radius: var(--border-radius-sm);
    margin-left: auto;
  }
  
  /* Style for checkbox */
  .column-toggle input[type="checkbox"] {
    appearance: none;
    -webkit-appearance: none;
    width: 16px;
    height: 16px;
    border: 1px solid var(--surface2);
    border-radius: 3px;
    outline: none;
    cursor: pointer;
    position: relative;
    background-color: var(--surface0);
  }
  
  .column-toggle input[type="checkbox"]:checked {
    background-color: var(--blue);
    border-color: var(--blue);
  }
  
  .column-toggle input[type="checkbox"]:checked::after {
    content: "";
    position: absolute;
    left: 5px;
    top: 2px;
    width: 4px;
    height: 8px;
    border: solid var(--crust);
    border-width: 0 2px 2px 0;
    transform: rotate(45deg);
  }
  
  .column-toggle input[type="checkbox"]:disabled {
    background-color: var(--surface1);
    border-color: var(--surface2);
    opacity: 0.6;
    cursor: not-allowed;
  }
  
  @media (max-width: 768px) {
    .column-settings-panel {
      width: calc(100% - 40px);
      max-width: 350px;
    }
  }
  
  @media (max-width: 576px) {
    th {
      padding: 0.5rem;
      font-size: 0.75rem;
    }
    
    .actions-header {
      width: 40px;
      min-width: 40px;
    }
    
    .column-settings-panel {
      right: 10px;
      top: 40px;
    }
  }
</style>
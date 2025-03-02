<script lang="ts">
  import type { Column } from "$lib/types";

  export let columns: Column[];

</script>

<thead>
  <tr>
    {#each columns.filter((col) => col.visible) as column}
      <th class="sortable">
        <div class="th-content">
          {column.label}
          <span
            class="sort-indicator"
          >
          </span>
        </div>
      </th>
    {/each}
    <th>Actions</th>
  </tr>
</thead>

<style>
  th {
    position: sticky;
    top: 0;
    background: var(--mantle);
    text-align: left;
    padding: 8px 12px;
    font-weight: 500;
    color: var(--subtext0);
    border-bottom: 1px solid var(--surface0);
    transition: background-color 0.2s ease;
    z-index: 3;
  }

  th:last-child {
    width: 120px;
    min-width: 120px;
    max-width: 120px;
  }

  .sortable {
    cursor: pointer;
    user-select: none;
  }

  .th-content {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .sort-indicator {
    color: var(--overlay0);
    font-size: 12px;
    opacity: 0.5;
    transition: all 0.2s ease;
  }

  .sort-indicator.active {
    color: var(--blue);
    opacity: 1;
  }

  .sortable:hover .sort-indicator {
    opacity: 1;
  }
</style>
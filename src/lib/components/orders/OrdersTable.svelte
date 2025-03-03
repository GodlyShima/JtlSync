<script lang="ts">
  import { processStore } from "$lib/stores/processes";
  import type { Column, VirtueMartOrder } from "$lib/types";
  import { onMount } from "svelte";
  import TableHeader from "./TableHeader.svelte";

  export let columns: Column[];
  export let orders: VirtueMartOrder[] = [];

  // Function to fetch orders
  async function fetchOrders() {
    try {
      processStore.setIsLoading(true);
      // In a real implementation, you'd call a Rust backend method to fetch orders
      // This is a placeholder - replace with actual backend method
      // const fetchedOrders = await TauriApiService.invoke<VirtueMartOrder[]>('get_synced_orders');
      // orders = fetchedOrders;
    } catch (error) {
      console.error('Failed to fetch orders:', error);
      processStore.update(state => ({ ...state, error: String(error) }));
    } finally {
      processStore.setIsLoading(false);
    }
  }

  // Mount effect to fetch orders when component loads
  onMount(fetchOrders);

  // Filtering and pagination logic
  $: filteredOrders = orders.filter(order => 
    columns
      .filter(col => col.visible)
      .some(col => 
        String(order[col.id])
          .toLowerCase()
          .includes(($processStore.searchTerm || '').toLowerCase())
      )
  );

  // Pagination
  $: {
    const itemsPerPage = $processStore.itemsPerPage || 15;
    const currentPage = $processStore.currentPage || 1;
    
    const startIndex = (currentPage - 1) * itemsPerPage;
    const endIndex = startIndex + itemsPerPage;
    
    paginatedOrders = filteredOrders.slice(startIndex, endIndex);
  }

  let paginatedOrders: VirtueMartOrder[] = [];
</script>

<div class="table-container">
  <table>
    <TableHeader {columns} />
    <tbody>
      {#each paginatedOrders as order}
        <tr>
          {#each columns.filter(col => col.visible) as column}
            <td>
              {column.format 
                ? column.format(order[column.id]) 
                : order[column.id] || ''}
            </td>
          {/each}
          <td class="actions">
            <div class="action-buttons">
            </div>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
  
  {#if paginatedOrders.length === 0}
    <div class="no-orders">
      {#if $processStore.isLoading}
        Laden...
      {:else}
        Keine Bestellungen gefunden
      {/if}
    </div>
  {/if}
</div>

<style>
  .table-container {
    flex: 1;
    overflow-x: auto;
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: var(--surface2) var(--mantle);
  }

  table {
    width: max-content;
    min-width: 100%;
    table-layout: fixed;
    border-collapse: collapse;
    font-size: 13px;
  }

  tbody tr {
    border-bottom: 1px solid var(--surface0);
    transition: background-color 0.2s ease;
  }

  tbody tr:hover {
    background-color: var(--surface0);
  }

  td {
    padding: 8px 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .actions {
    width: 120px;
    max-width: 120px;
  }

  .action-buttons {
    display: flex;
    gap: 8px;
    justify-content: center;
  }

  .btn-action {
    background: var(--surface1);
    border: none;
    border-radius: 4px;
    padding: 4px 8px;
    cursor: pointer;
    transition: background-color 0.2s ease;
  }

  .btn-action:hover {
    background: var(--surface2);
  }

  .no-orders {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 200px;
    color: var(--subtext0);
    font-style: italic;
  }
</style>
<script lang="ts">
  import { processStore } from "$lib/stores/processes";
  import type { Column, VirtueMartOrder } from "$lib/types";
  import { faEye, faSync, faTimes } from "@fortawesome/free-solid-svg-icons";
  import { onMount } from "svelte";
  import Fa from "svelte-fa";
  import TableHeader from "./TableHeader.svelte";

  export let columns: Column[];
  export let orders: VirtueMartOrder[] = [];

  // Order details modal state
  let showDetails = false;
  let selectedOrder: VirtueMartOrder | null = null;

  // Filtering and pagination logic
  $: filteredOrders = orders.filter(order => 
    columns
      .filter(col => col.visible)
      .some(col => 
        String(order[col.id as keyof VirtueMartOrder] || '')
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

  // Show order details
  function showOrderDetails(order: VirtueMartOrder) {
    selectedOrder = order;
    showDetails = true;
  }

  // Close order details modal
  function closeDetails() {
    showDetails = false;
    selectedOrder = null;
  }
  
  // Format date string
  function formatDate(dateStr: string): string {
    if (!dateStr) return '';
    
    const date = new Date(dateStr);
    return new Intl.DateTimeFormat('de-DE', { 
      year: 'numeric', 
      month: '2-digit', 
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit'
    }).format(date);
  }
  
  // Format price
  function formatPrice(price: number): string {
    return new Intl.NumberFormat('de-DE', { 
      style: 'currency', 
      currency: 'EUR' 
    }).format(price);
  }
  
  // Get order status text and class
  function getOrderStatusInfo(status: string | undefined) {
    if (!status) return { text: 'Unbekannt', class: 'unknown' };
    
    switch (status) {
      case 'P':
        return { text: 'Nicht bezahlt', class: 'pending' };
      case 'C':
        return { text: 'Bezahlt', class: 'success' };
      case 'X':
        return { text: 'Storniert', class: 'canceled' };
      default:
        return { text: status, class: 'unknown' };
    }
  }
  
  // Mount effect to update store when orders change
  onMount(() => {
    processStore.update(state => ({
      ...state,
      isLoading: false
    }));
    
    return () => {
      // Cleanup if needed
    };
  });
</script>

<div class="table-container">
  {#if showDetails && selectedOrder}
    <div class="order-details-modal">
      <div class="modal-content">
        <div class="modal-header">
          <h3>Bestelldetails #{selectedOrder.order_number}</h3>
          <button class="close-button" on:click={closeDetails}>
            <Fa icon={faTimes} />
          </button>
        </div>
        
        <div class="modal-body">
          <div class="details-grid">
            <div class="detail-section">
              <h4>Bestellinformationen</h4>
              <div class="detail-row">
                <span class="detail-label">Bestellnummer:</span>
                <span class="detail-value">{selectedOrder.order_number}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Datum:</span>
                <span class="detail-value">{formatDate(selectedOrder.created_on)}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Status:</span>
                <span class="detail-value">
                  <span class="status-badge {getOrderStatusInfo(selectedOrder.order_status).class}">
                    {getOrderStatusInfo(selectedOrder.order_status).text}
                  </span>
                </span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Gesamtbetrag:</span>
                <span class="detail-value">{formatPrice(selectedOrder.order_total)}</span>
              </div>
            </div>
            
            <div class="detail-section">
              <h4>Kundeninformationen</h4>
              <div class="detail-row">
                <span class="detail-label">Name:</span>
                <span class="detail-value">{selectedOrder.first_name} {selectedOrder.last_name}</span>
              </div>
              {#if selectedOrder.company}
                <div class="detail-row">
                  <span class="detail-label">Firma:</span>
                  <span class="detail-value">{selectedOrder.company}</span>
                </div>
              {/if}
              <div class="detail-row">
                <span class="detail-label">E-Mail:</span>
                <span class="detail-value">{selectedOrder.email || '-'}</span>
              </div>
              <div class="detail-row">
                <span class="detail-label">Telefon:</span>
                <span class="detail-value">{selectedOrder.phone_1 || '-'}</span>
              </div>
            </div>
            
            <div class="detail-section">
              <h4>Adresse</h4>
              <div class="detail-row">
                <span class="detail-label">Straße:</span>
                <span class="detail-value">{selectedOrder.address_1 || '-'}</span>
              </div>
              {#if selectedOrder.address_2}
                <div class="detail-row">
                  <span class="detail-label">Adresszusatz:</span>
                  <span class="detail-value">{selectedOrder.address_2}</span>
                </div>
              {/if}
              <div class="detail-row">
                <span class="detail-label">PLZ/Ort:</span>
                <span class="detail-value">{selectedOrder.zip} {selectedOrder.city}</span>
              </div>
            </div>
            
            <div class="detail-section">
              <h4>Notizen</h4>
              <div class="detail-row">
                <span class="detail-value notes">{selectedOrder.customer_note || 'Keine Notizen vorhanden'}</span>
              </div>
            </div>
          </div>
        </div>
        
        <div class="modal-footer">
          <button class="cancel-btn" on:click={closeDetails}>Schließen</button>
          <button class="primary-btn">
            <Fa icon={faSync} />
            Erneut synchronisieren
          </button>
        </div>
      </div>
    </div>
  {/if}

  <table class="data-table">
    <TableHeader {columns} />
    <tbody>
      {#if paginatedOrders.length === 0 && $processStore.isLoading}
        <tr class="loading-row">
          <td colspan={columns.filter(col => col.visible).length + 1} class="loading-cell">
            <div class="loading-spinner"></div>
            <span>Bestellungen werden geladen...</span>
          </td>
        </tr>
      {:else if paginatedOrders.length === 0}
        <tr class="empty-row">
          <td colspan={columns.filter(col => col.visible).length + 1} class="empty-cell">
            {$processStore.searchTerm ? 'Keine passenden Bestellungen gefunden' : 'Keine Bestellungen vorhanden'}
          </td>
        </tr>
      {:else}
        {#each paginatedOrders as order (order.virtuemart_order_id)}
          <tr class="data-row">
            {#each columns.filter(col => col.visible) as column}
              <td class="data-cell">
                {#if column.id === 'created_on'}
                  {formatDate(order.created_on)}
                {:else if column.id === 'order_total'}
                  {formatPrice(order.order_total)}
                {:else if column.id === 'order_status'}
                  <span class="status-badge {getOrderStatusInfo(order.order_status).class}">
                    {getOrderStatusInfo(order.order_status).text}
                  </span>
                {:else}
                  {column.format 
                    ? column.format(order[column.id as keyof VirtueMartOrder]) 
                    : order[column.id as keyof VirtueMartOrder] || '—'}
                {/if}
              </td>
            {/each}
            <td class="actions-cell">
              <div class="action-buttons">
                <button class="action-btn view" on:click={() => showOrderDetails(order)} title="Details anzeigen">
                  <Fa icon={faEye} />
                </button>
                <button class="action-btn sync" title="Erneut synchronisieren">
                  <Fa icon={faSync} />
                </button>
              </div>
            </td>
          </tr>
        {/each}
      {/if}
    </tbody>
  </table>
</div>

<style>
  .table-container {
    position: relative;
    flex: 1;
    overflow-x: auto;
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: var(--surface2) var(--mantle);
    background-color: var(--mantle);
    border-radius: var(--border-radius-sm);
  }

  .data-table {
    width: 100%;
    min-width: 100%;
    border-collapse: collapse;
    font-size: 0.8rem;
  }

  .data-row {
    border-bottom: 1px solid var(--surface0);
    transition: background-color var(--transition-fast);
  }

  .data-row:hover {
    background-color: var(--surface0);
  }

  .data-cell {
    padding: 0.6rem 0.8rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .actions-cell {
    width: 90px;
    max-width: 90px;
  }

  .action-buttons {
    display: flex;
    gap: 0.25rem;
    justify-content: center;
  }

  .action-btn {
    background: var(--surface0);
    border: none;
    border-radius: var(--border-radius-sm);
    width: 28px;
    height: 28px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text);
    transition: all var(--transition-fast);
  }

  .action-btn:hover {
    background: var(--surface1);
  }
  
  .action-btn.view:hover {
    background-color: var(--blue);
    color: var(--crust);
  }
  
  .action-btn.sync:hover {
    background-color: var(--green);
    color: var(--crust);
  }

  .loading-row, .empty-row {
    height: 200px;
  }
  
  .loading-cell, .empty-cell {
    text-align: center;
    color: var(--subtext0);
    font-style: italic;
    padding: 2rem;
  }
  
  .loading-cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
  }
  
  .loading-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--surface1);
    border-top-color: var(--blue);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  
  .status-badge {
    display: inline-block;
    padding: 0.2rem 0.5rem;
    border-radius: var(--border-radius-sm);
    font-size: 0.7rem;
    font-weight: 500;
  }
  
  .status-badge.success {
    background-color: rgba(var(--green-rgb), 0.2);
    color: var(--green);
  }
  
  .status-badge.pending {
    background-color: rgba(var(--yellow-rgb), 0.2);
    color: var(--yellow);
  }
  
  .status-badge.canceled {
    background-color: rgba(var(--red-rgb), 0.2);
    color: var(--red);
  }
  
  .status-badge.unknown {
    background-color: var(--surface1);
    color: var(--text);
  }
  
  /* Order Details Modal */
  .order-details-modal {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 1rem;
  }
  
  .modal-content {
    background-color: var(--mantle);
    border-radius: var(--border-radius-md);
    width: 100%;
    max-width: 800px;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: var(--shadow-lg);
    display: flex;
    flex-direction: column;
  }
  
  .modal-header {
    padding: 1rem;
    border-bottom: 1px solid var(--surface0);
    display: flex;
    justify-content: space-between;
    align-items: center;
    position: sticky;
    top: 0;
    background-color: var(--mantle);
    z-index: 1;
  }
  
  .modal-header h3 {
    margin: 0;
    font-size: 1.2rem;
    color: var(--blue);
  }
  
  .close-button {
    background: none;
    border: none;
    color: var(--text);
    cursor: pointer;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--border-radius-sm);
    transition: all var(--transition-fast);
  }
  
  .close-button:hover {
    background-color: var(--surface1);
  }
  
  .modal-body {
    padding: 1rem;
    overflow-y: auto;
  }
  
  .modal-footer {
    padding: 1rem;
    border-top: 1px solid var(--surface0);
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    position: sticky;
    bottom: 0;
    background-color: var(--mantle);
  }
  
  .details-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1.5rem;
  }
  
  .detail-section {
    margin-bottom: 1rem;
  }
  
  .detail-section h4 {
    margin: 0 0 0.5rem 0;
    font-size: 0.9rem;
    color: var(--blue);
    border-bottom: 1px solid var(--surface1);
    padding-bottom: 0.25rem;
  }
  
  .detail-row {
    display: flex;
    margin-bottom: 0.5rem;
    font-size: 0.85rem;
  }
  
  .detail-label {
    font-weight: 500;
    color: var(--subtext0);
    width: 120px;
    flex-shrink: 0;
  }
  
  .detail-value {
    color: var(--text);
  }
  
  .detail-value.notes {
    white-space: pre-line;
  }
  
  .cancel-btn {
    padding: 0.5rem 1rem;
    background-color: var(--surface1);
    color: var(--text);
    border: none;
    border-radius: var(--border-radius-sm);
    cursor: pointer;
    font-size: 0.85rem;
    transition: background-color var(--transition-fast);
  }
  
  .cancel-btn:hover {
    background-color: var(--surface2);
  }
  
  .primary-btn {
    padding: 0.5rem 1rem;
    background-color: var(--blue);
    color: var(--crust);
    border: none;
    border-radius: var(--border-radius-sm);
    cursor: pointer;
    font-size: 0.85rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    transition: background-color var(--transition-fast);
  }
  
  .primary-btn:hover {
    background-color: var(--sapphire);
  }
  
  /* Responsive styles */
  @media (max-width: 768px) {
    .details-grid {
      grid-template-columns: 1fr;
    }
    
    .data-cell {
      padding: 0.5rem;
    }
    
    /* Hide less important columns on small screens */
    .data-table [data-column="virtuemart_order_id"],
    .data-table [data-column="virtuemart_user_id"] {
      display: none;
    }
    
    .action-buttons {
      flex-direction: column;
      gap: 0.5rem;
    }
  }
  
  @media (max-width: 576px) {
    .modal-content {
      height: 100%;
      max-height: 100%;
      border-radius: 0;
    }
    
    .actions-cell {
      width: 40px;
    }
    
    .action-btn.sync {
      display: none;
    }
    
    .detail-row {
      flex-direction: column;
    }
    
    .detail-label {
      width: 100%;
      margin-bottom: 0.25rem;
    }
  }
</style>
<script lang="ts">
  import OrdersTable from "$lib/components/orders/OrdersTable.svelte";
  import ShopManager from "$lib/components/ShopManager.svelte";
  import { StatsBar } from "$lib/components/stats";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import ToolBar from "$lib/components/toolbar/ToolBar.svelte";
  import { column_definitions } from "$lib/definitions";
  import { TauriApiService } from "$lib/services/TauriApiService";
  import { processStore } from "$lib/stores/processes";
  import type { VirtueMartOrder } from "$lib/types";
  import { onMount } from "svelte";

  // Reactive store state destructuring
  $: ({
    error,
    searchTerm,
    isLoading,
    currentPage,
    itemsPerPage,
    isFrozen
  } = $processStore);

  // Columns configuration
  $: columns = column_definitions.map((col) => ({
    ...col,
    visible: col.required || col.visible
  }));

  // Orders state
  let orders: VirtueMartOrder[] = [];

  // Fetch orders from backend
  async function fetchSyncedOrders() {
    try {
      processStore.setIsLoading(true);
      
      // Listen to sync events to update orders
      const unlisten = await TauriApiService.listen('synced-orders', (event) => {
        orders = event.payload || [];
      });

      // Trigger backend to fetch synced orders
      await TauriApiService.invoke('get_synced_orders');
    } catch (err) {
      processStore.setError(String(err));
    } finally {
      processStore.setIsLoading(false);
    }
  }

  // Mount lifecycle hook
  onMount(() => {
    fetchSyncedOrders();
  });
</script>

<div class="app-container">
  <TitleBar />
  <main>
    <StatsBar />
    <ShopManager />

    <ToolBar 
      bind:searchTerm 
      bind:itemsPerPage
      bind:currentPage
      bind:refreshRate={$processStore.refreshRate}
      bind:isFrozen
      totalPages={Math.ceil(orders.length / itemsPerPage)}
      totalResults={orders.length}
    />

    {#if error}
      <div class="alert error">{error}</div>
    {/if}

    <OrdersTable {columns} bind:orders />
  </main>
</div>

<style>
  :global(:root) {
    --base: #1e1e2e;
    --mantle: #181825;
    --crust: #11111b;
    --text: #cdd6f4;
    --subtext0: #a6adc8;
    --subtext1: #bac2de;
    --surface0: #313244;
    --surface1: #45475a;
    --surface2: #585b70;
    --overlay0: #6c7086;
    --overlay1: #7f849c;
    --blue: #89b4fa;
    --lavender: #b4befe;
    --sapphire: #74c7ec;
    --sky: #89dceb;
    --red: #f38ba8;
    --maroon: #eba0ac;
    --peach: #fab387;
    --yellow: #f9e2af;
    --green: #a6e3a1;
    --teal: #94e2d5;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial,
      sans-serif, "Apple Color Emoji", "Segoe UI Emoji";
    background-color: var(--base);
    color: var(--text);
    -webkit-font-smoothing: antialiased;
    overflow: hidden;
    user-select: none;
  }

  main {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: min-content;
    overflow: hidden;
  }

  .app-container {
    height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .alert.error {
    background-color: var(--red);
    color: var(--base);
    padding: 10px;
    text-align: center;
  }
</style>
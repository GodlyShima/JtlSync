<script lang="ts">
  import type { Column, VirtueMartOrder } from "$lib/types";
  import TableHeader from "./TableHeader.svelte";

  export let orders: VirtueMartOrder[];
  export let columns: Column[];

</script>

<div class="table-container">
  <table>
    <TableHeader {columns} />
    <tbody>
    </tbody>
  </table>
</div>

<style>
  .table-container {
    flex: 1;
    overflow-x: auto;
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: var(--surface2) var(--mantle);
  }

  .table-container::-webkit-scrollbar {
    width: 8px;
    height: 8px;
  }

  .table-container::-webkit-scrollbar-track {
    background: var(--mantle);
    border-radius: 4px;
  }

  .table-container::-webkit-scrollbar-thumb {
    background: var(--surface2);
    border-radius: 4px;
    transition: background 0.2s ease;
  }

  .table-container::-webkit-scrollbar-thumb:hover {
    background: var(--surface1);
  }

  .table-container::-webkit-scrollbar-corner {
    background: var(--mantle);
  }

  table {
    width: max-content;
    min-width: 100%;
    table-layout: fixed;
    border-collapse: collapse;
    font-size: 13px;
  }
</style>
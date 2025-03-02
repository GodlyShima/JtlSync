<script lang="ts">
  import AppInfo from "../AppInfo.svelte";
  import PaginationControls from "./PaginationControls.svelte";
  import RefreshControls from "./RefreshControls.svelte";
  import SearchBox from "./SearchBox.svelte";

	export let searchTerm: string;
  export let itemsPerPage: number;
  export let currentPage: number;
  export let totalPages: number;
  export let totalResults: number;

  export let refreshRate: number;
  export let isFrozen: boolean;
</script>

<div class="toolbar">
  <div class="toolbar-content">
    <SearchBox bind:searchTerm />
    <div class="toolbar-spacer"></div>

    <PaginationControls 
      bind:itemsPerPage
      bind:currentPage
      {totalPages}
      {totalResults}
    />

    <div class="toolbar-spacer"></div>

    <RefreshControls bind:refreshRate bind:isFrozen />


    <AppInfo />
  </div>
</div>

<style>
  .toolbar {
    padding: 8px;
    border-bottom: 1px solid var(--surface0);
    background-color: var(--mantle);
  }

  .toolbar-content {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 12px;
    min-width: max-content;
  }

  .toolbar-spacer {
    flex: 1;
  }
</style>
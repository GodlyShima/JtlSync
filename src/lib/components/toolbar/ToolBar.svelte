<script lang="ts">
  import AppInfo from "../AppInfo.svelte";
  import SearchBox from "./SearchBox.svelte";

	export let searchTerm: string;
</script>

<div class="toolbar">
  <div class="toolbar-content">
    <SearchBox bind:searchTerm />
    <div class="toolbar-spacer"></div>

    <div class="toolbar-spacer"></div>

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
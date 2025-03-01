<script lang="ts">
  import { StatsBar } from "$lib/components/stats";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import ToolBar from "$lib/components/toolbar/ToolBar.svelte";
  import { processStore } from "$lib/stores/processes";

   $: ({
    error,
    searchTerm,
    isLoading,
    currentPage,
  } = $processStore);

</script>

<div class="app-container">
  <TitleBar />
  <main>
    <StatsBar />

    <ToolBar bind:searchTerm={$processStore.searchTerm} />
    <p>test</p>
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
</style>

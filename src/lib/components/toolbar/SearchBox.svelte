<script lang="ts">
  export let searchTerm: string;
</script>

<div class="search-box">
  <div class="search-input-wrapper">
    <input
      type="text"
      placeholder="Suchen"
      bind:value={searchTerm}
      class="search-input"
    />
    {#if searchTerm}
      <button class="btn-clear" on:click={() => (searchTerm = "")}>
        Löschen
      </button>
    {/if}
  </div>
</div>

<style>
  .search-box {
    width: 240px;
  }

  .search-input-wrapper {
    position: relative;
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .search-input {
    width: 100%;
    height: 28px;
    padding: 0 12px;
    padding-right: 70px;
    border: 1px solid var(--surface1);
    border-radius: 6px;
    font-size: 12px;
    background-color: var(--surface0);
    color: var(--text);
    transition: all 0.2s ease;
  }

  .btn-clear {
    position: absolute;
    right: 4px;
    top: 50%;
    transform: translateY(-50%);
    padding: 4px 8px;
    font-size: 11px;
    color: var(--subtext0);
    background: var(--surface1);
    border: none;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-clear:hover {
    background: var(--surface2);
    color: var(--text);
  }

  .search-input::-webkit-search-cancel-button {
    -webkit-appearance: none;
    height: 14px;
    width: 14px;
    margin-right: 4px;
    background: var(--overlay0);
    -webkit-mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cline x1='18' y1='6' x2='6' y2='18'%3E%3C/line%3E%3Cline x1='6' y1='6' x2='18' y2='18'%3E%3C/line%3E%3C/svg%3E")
      no-repeat 50% 50%;
    cursor: pointer;
  }

  .search-input::-webkit-search-cancel-button:hover {
    background: var(--text);
  }

  .search-input:hover {
    background-color: var(--surface1);
  }

  .search-input:focus {
    outline: none;
    border-color: var(--blue);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--blue) 25%, transparent);
  }
</style>
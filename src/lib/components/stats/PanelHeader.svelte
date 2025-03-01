<script lang="ts">
  import type { IconDefinition } from "@fortawesome/free-solid-svg-icons";
  import Fa from "svelte-fa";

  export let icon: IconDefinition;
  export let title: string;
  export let usageValue: string | null = null;
</script>

<div class="panel-header">
  <Fa {icon} />
  <h3>{title}</h3>
  {#if usageValue}
    <div class="usage-pill">{usageValue}</div>
  {/if}
</div>

<style>
  .panel-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid var(--surface0);
    flex-shrink: 0;
  }

  .panel-header h3 {
    font-size: 0.8rem;
    font-weight: 600;
    margin: 0;
    color: var(--text);
  }

  .panel-header :global(svg) {
    color: var(--blue);
    width: 0.8rem;
    height: 0.8rem;
  }

  .usage-pill {
    margin-left: auto;
    background: var(--surface0);
    padding: 0.15rem 0.5rem;
    border-radius: 1rem;
    font-size: 0.7rem;
    font-weight: 500;
  }
</style>
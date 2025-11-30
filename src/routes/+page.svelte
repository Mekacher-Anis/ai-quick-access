<script lang="ts">
  import { Textarea } from "$lib/components/ui/textarea";

  let searchValue = $state("");
</script>

<main class="container">
  <Textarea
    placeholder="Search..."
    bind:value={searchValue}
    class="floating-input"
  />
</main>

<style>
  .container {
    margin: 0;
    padding: 8px;
    min-height: 100vh;
    display: flex;
    background: transparent;
  }

  :global(.floating-input) {
    flex: 1;
    width: 100%;
    height: 100%;
    resize: none;
    border-radius: 12px;
    background: var(--card);
    border: 1px solid var(--border);
    box-shadow: 0 8px 10px rgba(0, 0, 0, 0.2);
    color: var(--foreground);
  }
</style>

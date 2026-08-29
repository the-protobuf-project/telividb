<!--
  The search panel: a form, an error, and results.

  It owns no rules. Everything about what a query needs and what a failure means
  lives in `state.svelte.ts`, which is why this file is a layout and little else.
-->
<script lang="ts">
  import { client } from "$lib/api";
  import Results from "$lib/ui/Results.svelte";
  import QueryForm from "./QueryForm.svelte";
  import { SearchState } from "./state.svelte";

  const state = new SearchState(client);

  $effect(() => {
    state.loadCollections();
  });
</script>

<div class="flex h-full flex-col gap-4 p-4">
  <QueryForm {state} onsubmit={() => state.run()} />

  {#if state.error}
    <p
      class="selectable rounded border border-bad/40 bg-bad/10 px-3 py-2 text-sm text-bad"
    >
      {state.error}
    </p>
  {/if}

  <Results results={state.results} />
</div>

<!--
  The model catalog.

  Curated rather than searchable, and the reason is worth stating in the window
  as well as the code: the engine loads a fixed set of GGUF architectures, and a
  model host carries tens of thousands of files that are mostly generative
  models it cannot read. A search box would mostly offer downloads that fail
  after several hundred megabytes.
-->
<script lang="ts">
  import { onDestroy } from "svelte";
  import { client } from "$lib/api";
  import * as Card from "$lib/components/ui/card";
  import { ModelsState } from "./state.svelte";
  import ModelRow from "./ModelRow.svelte";

  const state = new ModelsState(client);

  $effect(() => {
    state.load();
  });

  // Without this a closed panel keeps polling forever, which is invisible until
  // the log fills with it.
  onDestroy(() => state.dispose());
</script>

<Card.Root class="border-border">
  <Card.Header>
    <Card.Title>Embedding models</Card.Title>
    <Card.Description>
      An embedding model turns text into vectors. Install one to search by
      meaning rather than by exact match — everything here is verified against
      its published checksum before it is used.
    </Card.Description>
  </Card.Header>

  <Card.Content class="p-0">
    {#if state.error}
      <p class="text-destructive px-4 py-3 text-sm">{state.error}</p>
    {/if}

    {#if state.loading && state.models.length === 0}
      <p class="text-muted-foreground px-4 py-6 text-sm">Reading the catalog…</p>
    {:else if state.models.length === 0}
      <p class="text-muted-foreground px-4 py-6 text-sm">
        This build offers no models.
      </p>
    {:else}
      {#each state.models as model (model.id)}
        <ModelRow
          {model}
          install={state.installs[model.id]}
          onInstall={(id) => state.install(id)}
          onCancel={(id) => state.cancel(id)}
        />
      {/each}
    {/if}
  </Card.Content>

  <Card.Footer class="text-muted-foreground border-border block border-t pt-4 text-xs">
    <!-- Said plainly rather than discovered. The engine loads its models at
         startup and holds them resident, so a freshly installed one is not live
         until it restarts. Implying otherwise would leave someone searching
         against a model that is on disk and not loaded. -->
    A newly installed model is used after the engine restarts.
  </Card.Footer>
</Card.Root>

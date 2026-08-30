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
  import { ModelsState } from "./state.svelte";
  import ModelRow from "./ModelRow.svelte";
  import { Empty, Notice, Paged, Pager, SearchField, Skeleton } from "$lib/ui";

  interface Props {
    /** Called when a model becomes resident, so the window can re-check what
     *  the engine can now do. Text search is refused without one. */
    oninstalled?: () => void;
  }

  let { oninstalled }: Props = $props();

  const state = new ModelsState(client, () => oninstalled?.());

  $effect(() => {
    state.load();
  });

  // Without this a closed panel keeps polling forever, which is invisible until
  // the log fills with it.
  onDestroy(() => state.dispose());

  /** Search across the fields a person actually remembers a model by. */
  const list = new Paged(
    () => state.models,
    (m) => [m.displayName, m.id, m.license, m.description],
    8,
  );
</script>

<div class="view one">
  <div class="page">
  <div class="page-top">
    <div class="page-inner">
      <div class="page-head">
        <h1>Models</h1>
        <div class="spacer" style="flex: 1"></div>
        <SearchField
          bind:value={list.query}
          noun="models"
          matched={list.matches.length}
          total={state.models.length}
        />
      </div>
      <p class="lede">
        An embedding model turns text into vectors. Install one to search by
        meaning rather than by exact match — everything here is verified against
        its published checksum before it is used.
      </p>
    </div>
  </div>

  <div class="page-scroll">
    <div class="page-inner">
      <!-- A failed call and an empty catalog are different facts, and only one of
           them is ever true. Showing both said "this build offers no models" when
           what happened was that the request never reached a server. -->
      {#if state.error}
        <Notice tone="error">
          {state.error} — the catalog could not be read, so what this build offers
          is unknown.
        </Notice>
      {:else if state.loading && state.models.length === 0}
        <!-- Blocked out at the height the rows will take, so the list does not
             jump under the cursor when it arrives. -->
        {#each [0, 1, 2] as i (i)}
          <div class="row"><div class="row-main"><Skeleton lines={3} last="40%" /></div></div>
        {/each}
      {:else if state.models.length === 0}
        <Empty>This build offers no models.</Empty>
      {:else}
        {#each list.rows as model (model.id)}
          <ModelRow
            {model}
            install={state.installs[model.id]}
            onInstall={(id) => state.install(id)}
            onCancel={(id) => state.cancel(id)}
          />
        {:else}
          <Empty>Nothing matches “{list.query}”.</Empty>
        {/each}

        {#if list.paged}
          <div style="display: flex; justify-content: flex-end">
            <Pager page={list.page} pages={list.pages} go={(d) => list.go(d)} />
          </div>
        {/if}
      {/if}
    </div>
  </div>
</div>
</div>

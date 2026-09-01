<!--
  The model catalogue.

  Curated rather than searchable at source, and the reason belongs on screen as
  well as in the code: the engine loads a fixed set of GGUF architectures, and a
  model host carries tens of thousands of files that are mostly generative models
  it cannot read. A search box over *that* would mostly offer downloads that fail
  after several hundred megabytes. The search here filters what is already known
  to load.
-->
<script lang="ts">
  import { onDestroy } from "svelte";
  import { client } from "$lib/api";
  import { Button, DataViewport, Empty, Notice, Page, Pager, Paged, SearchField, Skeleton, Tag } from "$lib/ui";
  import { ModelsState } from "./state.svelte";
  import ModelRow from "./ModelRow.svelte";

  interface Props {
    /** Called when a model becomes resident, so the shell can re-check what the
     *  engine can now do. Text is refused without one. */
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

  const list = new Paged(
    () => state.models,
    (m) => [m.displayName, m.id, m.license, m.description],
    8,
  );

  let resident = $derived(state.models.find((m) => m.resident));
</script>


<!--
  On the shared page frame, like every other panel.

  This panel spent a revision building its own header and centring the list on a
  58rem column, which put its heading 100px from its column start where all six
  siblings put theirs at one `--gutter` — a fifth heading position in a window
  that should have one. `deviations.css` sets `.page-inner { max-width: none }`
  deliberately: the window is not a document and it has the width. `Page` is
  what makes that true here without this file restating any of it.

  The rows are the one thing allowed to scroll, and the pager is pinned under
  them rather than following them down — a control that travels off the bottom
  of a long list is a control you have to scroll to reach.
-->
<div class="view one">
  <Page
    title="Models"
    lede="An embedding model turns text into vectors. Everything here is verified against its published checksum before it is used."
  >
    {#snippet actions()}
      <!-- What is loaded, beside the heading rather than buried in a row: it is
           the one fact that decides whether text can be stored at all. -->
      {#if resident}
        <Tag tone="green">{resident.displayName} in memory</Tag>
      {:else}
        <Tag>no model loaded</Tag>
      {/if}

      <SearchField
        bind:value={list.query}
        noun="models"
        matched={list.matches.length}
        total={state.models.length}
      />
    {/snippet}

    <div class="models-stack">
      {#if state.error}
        <Notice tone="error">
          {state.error} — the catalogue could not be read, so what this build
          offers is unknown.
        </Notice>
      {:else if state.loading && state.models.length === 0}
        <!-- Blocked out at the height the rows will take, so the list does not
             jump under the cursor when it arrives. -->
        <div class="models-rows">
          {#each [0, 1, 2] as i (i)}
            <div class="row"><div class="row-main"><Skeleton lines={3} last="40%" /></div></div>
          {/each}
        </div>
      {:else if list.rows.length === 0}
        {#if list.query.trim()}
          <Empty>
            Nothing matches “{list.query}”.
            <Button variant="ghost" size="sm" onclick={() => (list.query = "")}>
              Clear search
            </Button>
          </Empty>
        {:else}
          <Empty>This build offers no models.</Empty>
        {/if}
      {:else}
        <DataViewport label="Models">
          {#each list.rows as model (model.id)}
            <ModelRow
              {model}
              install={state.installs[model.id]}
              onInstall={(id) => state.install(id)}
              onCancel={(id) => state.cancel(id)}
            />
          {/each}
        </DataViewport>

        {#if list.paged}
          <Pager page={list.page} pages={list.pages} go={(d) => list.go(d)} />
        {/if}
      {/if}
    </div>
  </Page>
</div>

<style>
  /* Rows take what is left; the pager keeps its own row so it stays on screen
     at every window height. */
  .models-stack {
    display: grid;
    grid-template-rows: 1fr auto;
    gap: calc(var(--u) * 3);
    min-height: 0;
    height: 100%;
  }

  /* A message is its own height. Only the list is entitled to the row it sits
     in — without this the error notice stretched its rule the full height of
     the pane, which reads as a giant empty box rather than as one sentence. */
  .models-stack > :global(.notice),
  .models-stack > :global(.empty) {
    align-self: start;
  }
</style>

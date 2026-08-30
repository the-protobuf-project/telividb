<!--
  What a collection holds, and how to put more in it.

  The table is the honest check on an import: a count in a toast says the call
  returned, and rows in a listing say the engine kept them.
-->
<script lang="ts">
  import { client } from "$lib/api";
  import Import from "./Import.svelte";
  import { Notice, Paged, Pager, SearchField } from "$lib/ui";
  import { PointsState } from "./state.svelte";

  interface Props {
    /** Collection to read and write. */
    collection: string;
    /** Whether the engine can turn text into a vector. */
    canEmbed: boolean;
  }

  let { collection, canEmbed }: Props = $props();

  const state = new PointsState(client);

  $effect(() => {
    state.collection = collection;
    state.refresh();
  });

  /**
   * A points table is the list most likely to be long — a collection is
   * thousands of rows, and 20 at a time is what fits without scrolling the
   * pane past its own header.
   */
  const list = new Paged(() => state.rows, (r) => [r.id, r.text], 20);
</script>

<div class="view one">
  <div class="page">
  <div class="page-top">
    <div class="page-inner">
      <div class="page-head">
        <h1>Points</h1>
        <div class="spacer"></div>
        <SearchField
          bind:value={list.query}
          noun="points"
          matched={list.matches.length}
          total={state.rows.length}
        />
        <span class="mono faint" style="font-size: 0.75rem">{collection || "—"}</span>
      </div>
      <Import {state} {canEmbed} />
    </div>
  </div>

  <div class="page-scroll">
    <div class="page-inner">
      {#if state.error}
        <Notice tone="error">{state.error}</Notice>
      {:else if state.imported !== null}
        <p class="hint">Imported <span class="mono">{state.imported}</span> point(s).</p>
      {/if}

      <!-- The table is the honest check on an import: a count says the call
           returned, rows say the engine kept them. -->
      <div class="table-wrap">
        <table>
          <thead>
            <tr><th style="width: 16rem">id</th><th>text</th></tr>
          </thead>
          <tbody>
            {#each list.rows as row (row.id)}
              <tr>
                <td class="cell-id mono selectable">{row.id}</td>
                <td class="cell-text selectable">{row.text ?? ""}</td>
              </tr>
            {:else}
              <tr>
                <td colspan="2" class="faint" style="text-align: center; padding: 2rem 0">
                  {#if list.query.trim()}
                    Nothing matches “{list.query}”.
                  {:else}
                    {collection ? "No points yet." : "No collection selected."}
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>

      {#if list.paged}
        <div style="display: flex; justify-content: flex-end">
          <Pager page={list.page} pages={list.pages} go={(d) => list.go(d)} />
        </div>
      {/if}
    </div>
  </div>
</div>
</div>

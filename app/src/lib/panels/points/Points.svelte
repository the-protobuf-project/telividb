<!--
  What a collection holds, and how to put more in it.

  The table is the honest check on an import: a count in a toast says the call
  returned, and rows in a listing say the engine kept them.
-->
<script lang="ts">
  import { client } from "$lib/api";
  import Import from "./Import.svelte";
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
</script>

<div class="page">
  <div class="page-top">
    <div class="page-inner">
      <div class="page-head">
        <h1>Points</h1>
        <div class="spacer"></div>
        <span class="mono faint" style="font-size: 0.75rem">{collection || "—"}</span>
      </div>
      <Import {state} {canEmbed} />
    </div>
  </div>

  <div class="page-scroll">
    <div class="page-inner">
      {#if state.error}
        <p class="selectable" style="color: var(--red-text)">{state.error}</p>
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
            {#each state.rows as row (row.id)}
              <tr>
                <td class="cell-id mono selectable">{row.id}</td>
                <td class="cell-text selectable">{row.text ?? ""}</td>
              </tr>
            {:else}
              <tr>
                <td colspan="2" class="faint" style="text-align: center; padding: 2rem 0">
                  {collection ? "No points yet." : "No collection selected."}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  </div>
</div>

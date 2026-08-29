<!--
  What a search answered.

  `complete` is rendered before the hits, not after. When a source could not be
  read, the honest reading of a short list is "this is partial", and a reader
  who sees the list first has already drawn the other conclusion.
-->
<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import type { SearchResponse } from "$lib/api";

  interface Props {
    /** The last answer, or null before the first query. */
    results: SearchResponse | null;
  }

  let { results = null }: Props = $props();
</script>

<div class="flex min-h-0 flex-1 flex-col gap-2">
  {#if results && !results.complete}
    <div
      class="border-destructive/40 bg-destructive/10 text-destructive rounded-lg border px-3 py-2 text-sm"
    >
      Partial results — not every source answered.
      {#if results.locked_vaults.length}
        Locked: <span class="selectable font-mono"
          >{results.locked_vaults.join(", ")}</span
        >
      {/if}
    </div>
  {/if}

  {#if results}
    <ScrollArea class="min-h-0 flex-1 rounded-lg border">
      {#each results.hits as hit (hit.id)}
        <article class="flex gap-3 border-b px-3 py-2 last:border-b-0">
          <Badge variant="secondary" class="tnum shrink-0">
            {hit.score.toFixed(3)}
          </Badge>
          <div class="min-w-0 flex-1">
            <p class="text-muted-foreground selectable truncate font-mono text-xs">
              {hit.id}
            </p>
            {#if hit.text}
              <p class="selectable mt-0.5 truncate text-sm">{hit.text}</p>
            {/if}
          </div>
        </article>
      {:else}
        <p class="text-muted-foreground px-3 py-8 text-center text-sm">
          Nothing matched.
        </p>
      {/each}
    </ScrollArea>

    <p class="text-muted-foreground text-xs">
      <span class="tnum">{results.hits.length}</span>
      {results.hits.length === 1 ? "hit" : "hits"}
    </p>
  {:else}
    <div
      class="flex flex-1 items-center justify-center rounded-lg border border-dashed"
    >
      <p class="text-muted-foreground text-sm">Run a search.</p>
    </div>
  {/if}
</div>

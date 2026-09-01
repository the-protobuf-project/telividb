<!--
  One exchange: the question, what was retrieved for it, and the answer.

  Retrieval is shown above the answer and never behind a disclosure, because the
  passages are the evidence — an answer whose support is a click away is one most
  readers will take on trust. The numbering here is the numbering the model was
  given, so a `[2]` in the prose points at the row labelled 2.

  Scores are shown for the same reason. A ranked list with no numbers cannot be
  judged: a 0.92 and a 0.31 look identical when both are simply "a result".
-->
<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import type { Exchange } from "./state.svelte";

  interface Props {
    /** The question, its passages, and the answer being written. */
    exchange: Exchange;
  }

  let { exchange }: Props = $props();

  /** Exactly what the model was shown, in the order it was numbered. */
  let passages = $derived(exchange.passages);
</script>

<div class="flex flex-col gap-3">
  <div class="bg-secondary/60 text-foreground ml-auto max-w-[85%] rounded-lg px-3 py-2 text-sm">
    {exchange.question}
  </div>

  {#if passages.length === 0}
    <p class="text-muted-foreground text-sm">
      Nothing to match against yet — this was the first thing stored.
    </p>
  {:else}
    <p class="text-muted-foreground text-xs tracking-wide uppercase">
      Retrieved {passages.length}
    </p>
    <div class="flex flex-col gap-2">
      {#each passages as passage, i (passage.id)}
        <div class="border-border flex items-start gap-3 rounded-lg border px-3 py-2">
          <span class="text-muted-foreground tnum shrink-0 text-xs">[{i + 1}]</span>
          <Badge variant="outline" class="tnum shrink-0 font-mono text-xs">
            {passage.score.toFixed(3)}
          </Badge>
          <span class="text-foreground min-w-0 text-sm">{passage.text}</span>
        </div>
      {/each}
    </div>
  {/if}

  {#if exchange.error}
    <!-- The refusal in full. "No key", "this space is key-wrapped" and "that
         model is gone" are different problems with different fixes, and a
         flattened message would send the reader to the wrong one. -->
    <p class="text-destructive selectable text-sm">{exchange.error}</p>
  {:else if exchange.text || exchange.streaming}
    <div class="text-foreground selectable text-sm whitespace-pre-wrap">
      {exchange.text}{#if exchange.streaming}<span
          class="bg-foreground ml-0.5 inline-block h-4 w-[2px] animate-pulse align-text-bottom"
        ></span>{/if}
    </div>
  {/if}
</div>

<!--
  One exchange: the question, and what came back.

  Scores are shown rather than hidden. This panel exists to make the embedding
  behaviour legible, and a ranked list with no numbers cannot be judged — a
  0.92 and a 0.31 look identical when both are simply "a result".
-->
<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import type { Exchange } from "./state.svelte";

  interface Props {
    /** The question and its answers. */
    exchange: Exchange;
  }

  let { exchange }: Props = $props();
</script>

<div class="flex flex-col gap-3">
  <div class="bg-secondary/60 text-foreground ml-auto max-w-[85%] rounded-lg px-3 py-2 text-sm">
    {exchange.question}
  </div>

  {#if exchange.hits.length === 0}
    <p class="text-muted-foreground text-sm">
      Nothing to match against yet — this was the first thing stored.
    </p>
  {:else}
    <div class="flex flex-col gap-2">
      {#each exchange.hits as hit (hit.id)}
        <div class="border-border flex items-start gap-3 rounded-lg border px-3 py-2">
          <Badge variant="outline" class="tnum shrink-0 font-mono text-xs">
            {hit.score.toFixed(3)}
          </Badge>
          <span class="text-foreground min-w-0 text-sm">
            <!-- The stored text, when the hit carries it. A bare id would be
                 accurate and useless: the whole question is what matched. -->
            {hit.text ?? hit.id}
          </span>
        </div>
      {/each}
    </div>
  {/if}
</div>

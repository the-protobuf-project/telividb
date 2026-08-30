<!--
  One exchange: what was asked, what was retrieved for it, and what was written
  back.

  Retrieval is a disclosure that starts **open when there is no answer yet** and
  closed once there is. Before an answer, the passages are the result; after it,
  they are the evidence — and evidence should be one click away rather than
  filling the screen above the thing it supports.
-->
<script lang="ts">
  import { Button } from "$lib/ui";
  import { enter } from "$lib/motion/motion";
  import type { Exchange } from "$lib/panels/ask/exchange.svelte";

  interface Props {
    /** The exchange to render. */
    turn: Exchange;
    /** The resident model's width, stated on every stored point. */
    dimensions?: number;
  }

  let { turn, dimensions }: Props = $props();

  let passages = $derived(turn.passages);

  /** The turn's own element, so it can arrive rather than appear. */
  let el = $state<HTMLElement | null>(null);
  /** Shown briefly after copying, so the click has an outcome. */
  let copied = $state(false);

  async function copy() {
    await navigator.clipboard.writeText(turn.text);
    copied = true;
    // Long enough to read, short enough not to linger into the next action.
    setTimeout(() => (copied = false), 1600);
  }

  // From below: a turn is appended to a thread that reads downward, so it
  // should come from the direction the next one will.
  $effect(() => {
    if (el) enter(el, "down");
  });
</script>

<div class="turn" bind:this={el}>
  <div class="said">{turn.question}</div>
  <div class="said-meta">
    <span>stored as a point</span>
    {#if dimensions}<span class="mono">{dimensions}d</span>{/if}
  </div>

  {#if passages.length > 0}
    <details class="retrieval" open={!turn.text}>
      <summary>
        <span>Retrieved {passages.length}</span>
        <span class="spacer" style="flex: 1"></span>
      </summary>
      <div class="retrieval-body">
        {#each passages as passage, i (passage.id)}
          <div class="hit">
            <span class="score">
              <!-- The citation number is the one the prompt used, so a `[2]` in
                   the answer points at the row marked 2 rather than at whatever
                   happens to be second on screen. -->
              <span class="cite" style="cursor: default">{i + 1}</span>
              <span class="meter"><i style="height: {Math.round(passage.score * 100)}%"></i></span>
              {passage.score.toFixed(3)}
            </span>
            <span class="hit-text">{passage.text}</span>
          </div>
        {/each}
      </div>
    </details>
  {/if}

  {#if turn.error}
    <!-- The refusal in full. "No key", "this space is key-wrapped" and "that
         model is gone" are different problems with different fixes. -->
    <p class="notice error selectable" role="alert">{turn.error}</p>
  {:else if turn.text || turn.streaming}
    <div class="answer selectable">
      {turn.text}{#if turn.streaming}<span class="answer-cursor"></span>{/if}
    </div>

    <!-- Under the answer rather than floating over it: these are things done
         *after* reading, and a control that covers the last line is in the way
         of the thing it acts on. -->
    <div class="answer-tools">
      {#if turn.streaming}
        <Button variant="ghost" size="sm" onclick={() => turn.stop()}>Stop</Button>
      {:else if turn.copyable}
        <Button variant="ghost" size="sm" onclick={copy}>
          {copied ? "Copied" : "Copy"}
        </Button>
      {/if}
    </div>
  {/if}
</div>

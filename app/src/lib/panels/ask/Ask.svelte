<!--
  Type a sentence, get back the closest things already said.

  The point of this panel is to make the embedding path visible with nothing
  else in the way: no file to map, no schema to fill in, no collection to name.
  Every question is also remembered, so the corpus builds itself and the
  neighbours get more interesting the more is in it.
-->
<script lang="ts">
  import { client } from "$lib/api";
  import { Button } from "$lib/components/ui/button";
  import { Textarea } from "$lib/components/ui/textarea";
  import { AskState } from "./state.svelte";
  import Answer from "./Answer.svelte";

  interface Props {
    /** Whether the engine can turn text into vectors yet. */
    canEmbed: boolean;
  }

  let { canEmbed }: Props = $props();

  const state = new AskState(client);

  /** Enter sends; shift-enter is a newline, as every chat box behaves. */
  function onkeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      state.ask();
    }
  }
</script>

<div class="mx-auto flex h-full w-full max-w-2xl flex-col gap-6 p-6">
  <div class="flex flex-col gap-3 pt-8">
    <div class="text-center">
      <h1 class="text-foreground text-xl font-medium">Ask anything</h1>
      <p class="text-muted-foreground mt-1 text-sm">
        Every sentence is embedded and kept. Ask again and the closest ones come
        back — matched by meaning, not by words.
      </p>
    </div>

    <Textarea
      bind:value={state.draft}
      {onkeydown}
      disabled={!canEmbed}
      rows={3}
      placeholder={canEmbed
        ? "Type a sentence and press Enter…"
        : "Waiting for an embedding model to finish loading…"}
      class="resize-none text-base"
    />

    <div class="flex items-center justify-between">
      <span class="text-muted-foreground text-xs">
        {#if !canEmbed}
          No model is loaded yet. Install one from Models, or give it a moment
          if the app has just started.
        {:else}
          Enter to send · Shift+Enter for a new line
        {/if}
      </span>
      <Button onclick={() => state.ask()} disabled={!state.canAsk || !canEmbed}>
        {state.asking ? "Thinking…" : "Ask"}
      </Button>
    </div>
  </div>

  {#if state.error}
    <p class="text-destructive text-sm">{state.error}</p>
  {/if}

  <div class="flex min-h-0 flex-1 flex-col gap-6 overflow-y-auto">
    {#each state.history as exchange, i (state.history.length - i)}
      <Answer {exchange} />
    {/each}

    {#if state.history.length === 0 && canEmbed}
      <p class="text-muted-foreground py-8 text-center text-sm">
        Nothing asked yet. The first sentence has nothing to match against —
        the second is where it starts being interesting.
      </p>
    {/if}
  </div>
</div>

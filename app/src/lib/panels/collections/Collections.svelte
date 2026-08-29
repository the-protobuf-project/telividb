<!--
  Creating a collection from a shipped preset.

  A preset rather than a schema editor because the engine takes a compiled
  descriptor set and never a `.proto` — a window cannot compile one, so without
  these a fresh install could search but never create anything to search.
-->
<script lang="ts">
  import { client } from "$lib/api";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Card from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import { CollectionState } from "./state.svelte";

  interface Props {
    /** Called with the new collection's id once one is created. */
    oncreated?: (collection: string) => void;
  }

  let { oncreated }: Props = $props();

  const state = new CollectionState(client);

  $effect(() => {
    state.loadPresets();
  });

  async function submit() {
    const name = await state.create();
    if (name) oncreated?.(state.collection);
  }
</script>

<div class="flex h-full flex-col gap-4 overflow-y-auto p-4">
  <div class="grid gap-3 sm:grid-cols-2">
    {#each state.presets as preset (preset.id)}
      <button
        type="button"
        onclick={() => (state.preset = preset.id)}
        class="rounded-lg border p-3 text-left transition-colors
               {state.preset === preset.id
          ? 'border-primary bg-accent'
          : 'hover:bg-accent/50'}"
      >
        <div class="flex items-center gap-2">
          <span class="text-sm font-medium">{preset.display_name}</span>
          <Badge variant="secondary" class="font-mono text-[10px]"
            >{preset.field}</Badge
          >
        </div>
        <p class="text-muted-foreground mt-1 text-xs">{preset.description}</p>
      </button>
    {/each}
  </div>

  <Card.Root>
    <Card.Header>
      <Card.Title class="text-sm">Name it</Card.Title>
      <Card.Description>
        The id forms the last segment of every point's resource name, and a
        resource name is permanent.
      </Card.Description>
    </Card.Header>
    <Card.Content class="flex items-start gap-2">
      <div class="flex-1">
        <Label for="collection-id" class="sr-only">Collection id</Label>
        <Input
          id="collection-id"
          bind:value={state.collection}
          placeholder="my-notes"
          class="font-mono"
          aria-invalid={state.idProblem !== null}
        />
        {#if state.idProblem}
          <p class="text-destructive mt-1.5 text-xs">{state.idProblem}</p>
        {/if}
      </div>
      <Button disabled={!state.ready || state.running} onclick={submit}>
        {state.running ? "Creating…" : "Create"}
      </Button>
    </Card.Content>
  </Card.Root>

  {#if state.error}
    <p
      class="selectable border-destructive/40 bg-destructive/10 text-destructive rounded-lg border px-3 py-2 text-sm"
    >
      {state.error}
    </p>
  {:else if state.created}
    <p class="text-muted-foreground text-sm">
      Created <span class="selectable font-mono">{state.created}</span>.
    </p>
  {/if}
</div>

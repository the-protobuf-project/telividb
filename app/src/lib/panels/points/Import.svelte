<!--
  Choosing a file and mapping its columns.

  Both pickers stay visible even when the guess is right: a wrong text column
  embeds the wrong thing, and the mistake is invisible afterwards — the points
  are written, they simply never match anything.
-->
<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select";
  import type { PointsState } from "./state.svelte";

  interface Props {
    /** The panel state this form reads and writes. */
    state: PointsState;
    /** Whether the engine can turn text into a vector. */
    canEmbed: boolean;
  }

  let { state, canEmbed }: Props = $props();

  async function choose(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    state.read(await file.text());
    // Cleared so choosing the same file twice re-reads it; without this the
    // second choice fires no change event and looks like nothing happened.
    input.value = "";
  }
</script>

<div class="flex flex-col gap-3 rounded-lg border p-3">
  {#if !canEmbed}
    <!--
      Said before a file is chosen rather than after the columns are mapped.
      The server refuses text without a model — for storage as much as for
      search — and discovering that at the end wastes everything the person
      did to get there.
    -->
    <p
      class="border-destructive/40 bg-destructive/10 text-destructive rounded-md border px-2.5 py-2 text-xs"
    >
      No embedding model is loaded yet, so this engine cannot accept text.
      Install one from <span class="font-medium">Models</span> — or wait a
      moment if you just started the app, since a model is read into memory in
      the background and is not ready the instant the window opens.
    </p>
  {/if}

  <div class="flex items-center gap-2">
    <Button
      variant="secondary"
      size="sm"
      disabled={!canEmbed}
      onclick={() => document.getElementById("csv-file")?.click()}
    >
      Choose a CSV
    </Button>
    <input
      id="csv-file"
      type="file"
      accept=".csv,text/csv"
      class="hidden"
      onchange={choose}
    />
    {#if state.parsed}
      <span class="text-muted-foreground text-xs">
        <span class="tnum">{state.importable}</span> of
        <span class="tnum">{state.parsed.rows.length}</span> rows have text
      </span>
    {/if}
  </div>

  {#if state.parsed}
    <div class="flex items-end gap-2">
      <div class="flex-1">
        <Label class="text-muted-foreground mb-1 text-xs">id column</Label>
        <Select.Root type="single" bind:value={state.mapping.id}>
          <Select.Trigger class="w-full">{state.mapping.id}</Select.Trigger>
          <Select.Content>
            {#each state.columns as column (column)}
              <Select.Item value={column} label={column}>{column}</Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>

      <div class="flex-1">
        <Label class="text-muted-foreground mb-1 text-xs">text column</Label>
        <Select.Root type="single" bind:value={state.mapping.text}>
          <Select.Trigger class="w-full">{state.mapping.text}</Select.Trigger>
          <Select.Content>
            {#each state.columns as column (column)}
              <Select.Item value={column} label={column}>{column}</Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </div>

      <Button
        disabled={!state.ready || state.running}
        onclick={() => state.importRows()}
      >
        {state.running ? "Importing…" : `Import ${state.importable}`}
      </Button>
    </div>
  {/if}
</div>

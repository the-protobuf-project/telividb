<!--
  The query controls.

  Presentational: it reads and writes the state it is handed and runs nothing.
  Whether a query is runnable is the state's rule, not this component's.

  Every control is a library component. A native `<select>` renders as the
  platform's own widget, which on macOS means a control that ignores the app's
  palette entirely and looks pasted in — and cannot be styled into agreement.
-->
<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select";
  import type { SearchState } from "./state.svelte";

  interface Props {
    /** The panel state this form reads and writes. */
    state: SearchState;
    /** Run the query. */
    onsubmit: () => void;
  }

  let { state, onsubmit }: Props = $props();
</script>

<form
  class="bg-card flex items-center gap-2 rounded-lg border p-2"
  onsubmit={(e) => {
    e.preventDefault();
    onsubmit();
  }}
>
  <Select.Root type="single" bind:value={state.collection}>
    <Select.Trigger class="w-44">
      {state.collection || "no collections"}
    </Select.Trigger>
    <Select.Content>
      {#each state.collections as collection (collection.id)}
        <Select.Item value={collection.id}>{collection.id}</Select.Item>
      {/each}
    </Select.Content>
  </Select.Root>

  <Input
    bind:value={state.field}
    placeholder="field"
    class="w-36 font-mono"
    aria-label="Vector field to search"
  />

  <Input
    bind:value={state.text}
    placeholder="query"
    class="flex-1"
    aria-label="Query text"
  />

  <Label class="text-muted-foreground gap-1.5 text-xs">
    k
    <Input
      type="number"
      bind:value={state.k}
      min="1"
      max="1000"
      class="tnum w-20"
      aria-label="Neighbours to return"
    />
  </Label>

  <Button type="submit" disabled={!state.ready || state.running}>
    {state.running ? "Searching…" : "Search"}
  </Button>
</form>

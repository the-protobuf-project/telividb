<!--
  What a collection holds, and how to put more in it.

  The table is the honest check on an import: a count in a toast says the call
  returned, and rows in a listing say the engine kept them.
-->
<script lang="ts">
  import { client } from "$lib/api";
  import * as Table from "$lib/components/ui/table";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
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

<div class="flex h-full flex-col gap-3 p-4">
  <Import {state} {canEmbed} />

  {#if state.error}
    <p
      class="selectable border-destructive/40 bg-destructive/10 text-destructive rounded-lg border px-3 py-2 text-sm"
    >
      {state.error}
    </p>
  {:else if state.imported !== null}
    <p class="text-muted-foreground text-sm">
      Imported <span class="tnum">{state.imported}</span> point(s).
    </p>
  {/if}

  <ScrollArea class="min-h-0 flex-1 rounded-lg border">
    <Table.Root>
      <Table.Header>
        <Table.Row>
          <Table.Head class="w-48">id</Table.Head>
          <Table.Head>text</Table.Head>
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {#each state.rows as row (row.id)}
          <Table.Row>
            <Table.Cell class="selectable font-mono text-xs">{row.id}</Table.Cell>
            <Table.Cell class="selectable truncate text-sm">
              {row.text ?? ""}
            </Table.Cell>
          </Table.Row>
        {:else}
          <Table.Row>
            <Table.Cell colspan={2} class="text-muted-foreground py-8 text-center text-sm">
              {collection === "" ? "Choose a collection." : "No points yet."}
            </Table.Cell>
          </Table.Row>
        {/each}
      </Table.Body>
    </Table.Root>
  </ScrollArea>
</div>

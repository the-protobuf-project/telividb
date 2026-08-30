<!--
  Make one thing: type a name, get an id.

  The id is derived from the display name and shown rather than hidden, because
  it is the half that cannot be changed afterwards — it becomes the last segment
  of the resource name. Showing it at the moment of creation is the only point
  where a person can still object to it.
-->
<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { suggestId } from "$lib/api";

  interface Props {
    /** What is being made, for the placeholder and the button. */
    noun: string;
    /** Whether a call is already in flight. */
    busy?: boolean;
    /** Called with the id and the display name. */
    create: (id: string, displayName: string) => void;
  }

  let { noun, busy = false, create }: Props = $props();

  let displayName = $state("");
  let id = $derived(suggestId(displayName));
  let ready = $derived(id.length > 0 && !busy);

  function submit() {
    if (!ready) return;
    create(id, displayName.trim());
    displayName = "";
  }
</script>

<div class="flex items-center gap-2">
  <Input
    bind:value={displayName}
    placeholder="New {noun}…"
    class="h-8 w-56 text-sm"
    aria-label="New {noun} name"
    onkeydown={(e) => {
      if (e.key === "Enter") submit();
    }}
  />
  {#if id}
    <span class="text-muted-foreground font-mono text-xs" title="The permanent id">
      {id}
    </span>
  {/if}
  <Button size="sm" disabled={!ready} onclick={submit}>Create</Button>
</div>

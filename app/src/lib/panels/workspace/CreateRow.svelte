<!--
  Make one thing: type a name, get an id.

  The id is derived from the display name and shown rather than hidden, because
  it is the half that cannot be changed afterwards — it becomes the last segment
  of the resource name. Creation is the only moment a person can still object.
-->
<script lang="ts">
  import { suggestId } from "$lib/api";

  interface Props {
    /** What is being made, for the placeholder and the button. */
    noun: string;
    /** Whether a call is already in flight. */
    busy?: boolean;
    /**
     * Called with the id and the display name.
     *
     * Resolves to whether the resource was actually created. The field is only
     * cleared on `true`: a refused name that vanished from the box left the
     * person retyping something the server had already rejected once.
     */
    create: (id: string, displayName: string) => Promise<boolean> | boolean;
  }

  let { noun, busy = false, create }: Props = $props();

  let displayName = $state("");
  let id = $derived(suggestId(displayName));
  let ready = $derived(id.length > 0 && !busy);

  async function submit() {
    if (!ready) return;
    const created = await create(id, displayName.trim());
    if (created) displayName = "";
  }
</script>

<div style="display: flex; gap: 0.5rem; align-items: center">
  <input
    class="input"
    style="width: 14rem"
    bind:value={displayName}
    placeholder="New {noun}…"
    aria-label="New {noun} name"
    onkeydown={(e) => {
      if (e.key === "Enter") submit();
    }}
  />
  {#if id}
    <span class="mono faint" style="font-size: 0.75rem" title="The permanent id">
      {id}
    </span>
  {/if}
  <button class="btn sm" type="button" disabled={!ready} onclick={submit}>
    Create
  </button>
</div>

<!--
  The gate: nothing exists until an organization does.

  Every other resource is named inside one — a project is
  `organizations/{org}/projects/{p}`, a space likewise — so there is no useful
  screen to show before one exists. Offering the dock here would be offering
  panels that can only say "nothing yet", which reads as a broken app rather
  than an empty one.

  A single field on purpose. The id is derived and shown because it becomes the
  permanent part of every name beneath it, and this is the only moment anyone
  can still object to it.
-->
<script lang="ts">
  import { client, suggestId } from "$lib/api";

  interface Props {
    /** Called with the created organization's resource name. */
    oncreated: (name: string) => void;
  }

  let { oncreated }: Props = $props();

  let displayName = $state("");
  let busy = $state(false);
  let error = $state<string | null>(null);

  let id = $derived(suggestId(displayName));
  let ready = $derived(id.length > 0 && !busy);

  async function create() {
    if (!ready) return;
    busy = true;
    error = null;
    try {
      const made = await client.createOrganization(id, displayName.trim());
      oncreated(made.name);
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      busy = false;
    }
  }
</script>

<div class="stage">
  <div class="card" style="max-width: 32rem; width: 100%">
    <div class="card-head">
      <h1>Name your organization</h1>
      <p class="lede">
        Everything is stored inside one — projects, spaces, collections and
        points all carry its name. You can add more later; this is just the
        first.
      </p>
    </div>

    <div class="card-body">
      <div class="field">
        <label class="label" for="org-name">Organization name</label>
        <input
          id="org-name"
          class="input"
          bind:value={displayName}
          placeholder="Acme Research"
          onkeydown={(e) => {
            if (e.key === "Enter") create();
          }}
        />
        <p class="hint">
          {#if id}
            Stored as <span class="mono">organizations/{id}</span>. This part is
            permanent — the name above it can change.
          {:else}
            The display name can change later. The id derived from it cannot.
          {/if}
        </p>
      </div>

      {#if error}
        <p class="selectable" style="color: var(--red-text)">{error}</p>
      {/if}
    </div>

    <div class="card-foot">
      <button class="btn" type="button" disabled={!ready} onclick={create}>
        {busy ? "Creating…" : "Create and continue"}
      </button>
    </div>
  </div>
</div>

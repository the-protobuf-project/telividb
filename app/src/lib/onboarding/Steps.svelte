<!--
  The four first-run steps, one pane at a time.

  Each pane is only the fields that step needs. The heading says what is being
  decided and the lede says why it cannot be changed later, where that is true —
  an id and a protection are both permanent, and the moment before they are
  committed is the only one where saying so helps.
-->
<script lang="ts">
  import { Field, Input, Row, Seg, Tag } from "$lib/ui";
  import type { Protection } from "$lib/ui";
  import type { Flow } from "./flow.svelte";

  interface Props {
    /** The first-run state. */
    flow: Flow;
  }

  let { flow }: Props = $props();

  const protections: readonly Protection[] = ["none", "private", "vault", "sealed"];

  /** Bytes as a person reads them. */
  function mb(bytes: number): string {
    return bytes >= 1_000_000_000
      ? `${(bytes / 1_000_000_000).toFixed(1)} GB`
      : `${Math.round(bytes / 1_000_000)} MB`;
  }
</script>

{#if flow.step === 0}
  <div>
    <h1>Install an embedding model</h1>
    <p class="lede">
      A model turns what you write into vectors, which is how this finds things by
      meaning rather than by matching words. Everything here is checked against
      its published checksum before it is used.
    </p>
  </div>

  {#if flow.hasModel}
    <p class="hint">A model is already loaded. This step is done.</p>
  {:else}
    <div style="display: flex; flex-direction: column; gap: calc(var(--u) * 2)">
      {#each flow.models.slice(0, 3) as model (model.id)}
        <Row
          name={model.displayName}
          selected={flow.model === model.id}
          onclick={() => (flow.model = model.id)}
        >
          {#snippet badges()}
            {#if model.recommended}<Tag tone="blue">recommended</Tag>{/if}
          {/snippet}
          {#snippet meta()}
            <div class="row-meta">{model.description}</div>
            <div class="row-meta mono">
              {model.dimensions} dimensions · {mb(model.sizeBytes)}
            </div>
          {/snippet}
        </Row>
      {/each}
    </div>
  {/if}
{:else if flow.step === 1}
  <div>
    <h1>Name your organization</h1>
    <p class="lede">
      Everything lives under an organization — projects, spaces and the
      collections behind them.
    </p>
  </div>
  <Field
    label="Organization"
    hint={flow.organization
      ? `organizations/${flow.idFor(flow.organization)} — this part is permanent`
      : "organizations/…"}
  >
    <Input bind:value={flow.organization} placeholder="Acme Research" />
  </Field>
{:else if flow.step === 2}
  <div>
    <h1>Create a project</h1>
    <p class="lede">
      A project is where work is grouped and where access is granted. You can add
      more at any time.
    </p>
  </div>
  <Field
    label="Project"
    hint={flow.project ? `projects/${flow.idFor(flow.project)}` : "projects/…"}
  >
    <Input bind:value={flow.project} placeholder="Retrieval" />
  </Field>
{:else}
  <div>
    <h1>Open a space</h1>
    <p class="lede">
      A space holds conversation and the points behind it. Its protection is fixed
      when it is created — it decides which segments the contents are routed to,
      so changing it later would mean rewriting all of them.
    </p>
  </div>
  <Field label="Space" hint={flow.space ? `spaces/${flow.idFor(flow.space)}` : "spaces/…"}>
    <Input bind:value={flow.space} placeholder="Notes" />
  </Field>
  <Field label="Protection" hint="Only a local model may answer from a vault or a sealed space.">
    <Seg options={protections} bind:value={flow.protection} label="Protection" />
  </Field>
{/if}

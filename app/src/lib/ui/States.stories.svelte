<!--
  Every state a control can be in, which the mock does not draw.

  The mock is a happy-path picture: every field filled, every list populated,
  nothing in flight. A window spends much of its life in one of these, and
  without them each panel invents its own — which is how "loading" ends up
  looking different on three screens.
-->
<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import { Button, Field, Input, Notice, PanelLabel, Row, Skeleton, Tag, Textarea } from "./index";

  const { Story } = defineMeta({ title: "Elements/States" });

  let name = $state("Acme Research!");
  let busy = $state(false);

  // Deliberately the rule the app uses: an id must survive being a path segment.
  let invalid = $derived(/[^a-zA-Z0-9 _-]/.test(name));

  async function submit() {
    busy = true;
    await new Promise((r) => setTimeout(r, 1600));
    busy = false;
  }
</script>

<Story name="Validation">
  <div style="max-width: 28rem; display: flex; flex-direction: column; gap: 1rem">
    <Field
      label="Organization"
      hint="The id derived from it is permanent."
      error={invalid ? "Letters, numbers, spaces, hyphens and underscores only — the id becomes a path segment." : undefined}
    >
      <Input bind:value={name} {invalid} placeholder="Acme Research" />
    </Field>
    <p class="hint">Type a <span class="mono">!</span> or a <span class="mono">/</span> to see it reject.</p>
  </div>
</Story>

<Story name="In flight">
  <div style="display: flex; gap: 0.5rem; align-items: center">
    <Button loading={busy} onclick={submit}>Create and continue</Button>
    <Button variant="ghost" loading={busy}>Ghost</Button>
    <Button size="sm" loading={busy}>Small</Button>
    <span class="hint">Click to watch it spin for a moment.</span>
  </div>
</Story>

<Story name="Loading">
  <div style="max-width: 46rem; display: flex; flex-direction: column; gap: 0.5rem">
    <PanelLabel>Waiting for the catalog</PanelLabel>
    {#each [0, 1, 2] as i (i)}
      <div class="row"><div class="row-main"><Skeleton lines={3} last="40%" /></div></div>
    {/each}
  </div>
</Story>

<Story name="Messages">
  <div style="max-width: 46rem; display: flex; flex-direction: column; gap: 0.5rem">
    <Notice>Retrieval always happens on this machine.</Notice>
    <Notice tone="ok">Model verified against its published checksum.</Notice>
    <Notice tone="warn">Running on sample data — Identity.ListUsers is not served yet.</Notice>
    <Notice tone="error">
      Journal is key-wrapped, so its contents are answered by a model on this
      machine rather than a remote one. OpenAI is remote.
    </Notice>
  </div>
</Story>

<Story name="Empty and refused">
  <div style="max-width: 46rem; display: flex; flex-direction: column; gap: 1rem">
    <div class="empty">No calls measured yet. Open Workspace and the timings appear here.</div>
    <Row name="Qwen3 Embedding 8B">
      {#snippet badges()}<Tag tone="amber">too large</Tag>{/snippet}
      {#snippet meta()}
        <div class="row-meta">Needs 8.1 GB of device memory; this machine reports a 4.0 GB budget.</div>
      {/snippet}
      {#snippet action()}<Button size="sm" disabled>Install</Button>{/snippet}
    </Row>
    <Textarea placeholder="Waiting for an embedding model to finish loading…" disabled rows={2} />
  </div>
</Story>

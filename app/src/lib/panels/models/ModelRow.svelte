<!--
  One model in the catalog.

  The size and the licence sit next to the install button on purpose: both are
  decisions a person should make before several hundred megabytes move, and a row
  that hides them behind a details view is a row that gets clicked blind.
-->
<script lang="ts">
  import { Bar, Button, Dot, Row, Tag } from "$lib/ui";
  import { isFinished } from "$lib/api";
  import type { CatalogModel, Installation } from "$lib/api";

  interface Props {
    /** The model this row describes. */
    model: CatalogModel;
    /** Its installation, when one has been started. */
    install?: Installation;
    /** Start installing. */
    onInstall: (id: string) => void;
    /** Stop installing. */
    onCancel: (id: string) => void;
  }

  let { model, install, onInstall, onCancel }: Props = $props();

  const running = $derived(install !== undefined && !isFinished(install.state));
  const fraction = $derived(
    install && install.totalBytes > 0 ? install.progressBytes / install.totalBytes : 0,
  );

  /** Bytes as a person reads them. */
  function megabytes(bytes: number): string {
    return bytes >= 1_000_000_000
      ? `${(bytes / 1_000_000_000).toFixed(1)} GB`
      : `${Math.round(bytes / 1_000_000)} MB`;
  }
</script>

<Row name={model.displayName}>
  {#snippet badges()}
    {#if model.resident}
      <Tag tone="green">in memory</Tag>
    {:else if model.installed}
      <Tag>installed</Tag>
    {/if}
    {#if model.recommended}<Tag tone="blue">recommended</Tag>{/if}
  {/snippet}

  {#snippet meta()}
    <div class="row-meta">{model.description}</div>
    <div class="row-meta mono">
      {model.dimensions} dim · {model.contextLength.toLocaleString()} tokens ·
      {megabytes(model.sizeBytes)} · {model.license}
    </div>
    {#if running}
      <!-- A determinate bar only once a total is known: one that fills from
           nothing to nothing reads as a stall rather than as a start. -->
      <div style="margin-top: calc(var(--u) * 2)"><Bar value={fraction} /></div>
      <div class="row-meta mono">
        {install?.state} · {megabytes(install?.progressBytes ?? 0)} of
        {megabytes(install?.totalBytes ?? 0)}
      </div>
    {/if}
  {/snippet}

  {#snippet action()}
    <a class="btn ghost sm" href={model.repositoryUri} target="_blank" rel="noreferrer">
      Source
    </a>
    {#if model.installed}
      <!-- The word as well as the mark: a green dot was the only thing saying
           this model is on disk, which is unreadable without colour. -->
      <span style="display: inline-flex; align-items: center; gap: calc(var(--u) * 1.5)">
        <Dot state="live" decorative />
        <span class="faint" style="font-size: 0.75rem">on disk</span>
      </span>
    {:else if running}
      <Button variant="ghost" size="sm" onclick={() => onCancel(model.id)}>Cancel</Button>
    {:else}
      <Button size="sm" onclick={() => onInstall(model.id)}>Install</Button>
    {/if}
  {/snippet}
</Row>

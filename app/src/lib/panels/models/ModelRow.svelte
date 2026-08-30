<!--
  One model in the catalog.

  The size and the licence sit next to the install button on purpose: both are
  decisions a person should make before several hundred megabytes move, and a row
  that hides them behind a details view is a row that gets clicked blind.
-->
<script lang="ts">
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
  const percent = $derived(
    install && install.totalBytes > 0
      ? Math.min(100, (install.progressBytes / install.totalBytes) * 100)
      : 0,
  );

  /** Bytes as a person reads them. */
  function megabytes(bytes: number): string {
    return bytes >= 1_000_000_000
      ? `${(bytes / 1_000_000_000).toFixed(1)} GB`
      : `${Math.round(bytes / 1_000_000)} MB`;
  }
</script>

<div class="row">
  <div class="row-main">
    <div class="row-title">
      <span class="row-name">{model.displayName}</span>
      {#if model.resident}
        <span class="tag green">in memory</span>
      {:else if model.installed}
        <span class="tag">installed</span>
      {/if}
      {#if model.recommended}
        <span class="tag blue">recommended</span>
      {/if}
    </div>
    <div class="row-meta">{model.description}</div>
    <div class="row-meta mono">
      {model.dimensions} dim · {model.contextLength.toLocaleString()} tokens ·
      {megabytes(model.sizeBytes)} · {model.license}
    </div>

    {#if running}
      <!-- A determinate bar only once a total is known: a bar that fills from
           nothing to nothing reads as a stall rather than as a start. -->
      <div class="bar" style="margin-top: 0.5rem">
        <i style="width: {percent}%"></i>
      </div>
      <div class="row-meta mono">
        {install?.state} · {megabytes(install?.progressBytes ?? 0)} of
        {megabytes(install?.totalBytes ?? 0)}
      </div>
    {/if}
  </div>

  <div style="display: flex; gap: 0.5rem; align-items: center; flex: none">
    <a class="btn ghost sm" href={model.repositoryUri} target="_blank" rel="noreferrer">
      Source
    </a>
    {#if model.installed}
      <span class="dot live" title="On disk"></span>
    {:else if running}
      <button class="btn ghost sm" type="button" onclick={() => onCancel(model.id)}>
        Cancel
      </button>
    {:else}
      <button class="btn sm" type="button" onclick={() => onInstall(model.id)}>
        Install
      </button>
    {/if}
  </div>
</div>

<!--
  One model in the catalog.

  The size and the licence sit next to the install button on purpose: both are
  decisions a person should make before several hundred megabytes move, and a
  row that hides them behind a details view is a row that gets clicked blind.
-->
<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { Progress } from "$lib/components/ui/progress";
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

<div class="border-border flex flex-col gap-3 border-b px-4 py-4 last:border-b-0">
  <div class="flex items-start justify-between gap-4">
    <div class="min-w-0">
      <div class="flex items-center gap-2">
        <span class="text-foreground font-medium">{model.displayName}</span>
        {#if model.recommended}
          <Badge variant="secondary">Recommended</Badge>
        {/if}
        {#if model.installed}
          <Badge variant="outline">Installed</Badge>
        {/if}
      </div>
      <p class="text-muted-foreground mt-1 text-sm">{model.description}</p>
      <p class="text-muted-foreground mt-2 font-mono text-xs">
        {model.dimensions} dimensions · {model.contextLength.toLocaleString()} tokens ·
        {megabytes(model.sizeBytes)} · {model.license}
      </p>
    </div>

    <div class="flex shrink-0 items-center gap-2">
      <Button variant="ghost" size="sm" href={model.repositoryUri} target="_blank">
        Source
      </Button>
      {#if model.installed}
        <Button variant="outline" size="sm" disabled>Installed</Button>
      {:else if running}
        <Button variant="outline" size="sm" onclick={() => onCancel(model.id)}>
          Cancel
        </Button>
      {:else}
        <Button size="sm" onclick={() => onInstall(model.id)}>Install</Button>
      {/if}
    </div>
  </div>

  {#if install}
    {#if running}
      <div class="flex items-center gap-3">
        <Progress value={percent} class="h-1.5" />
        <span class="text-muted-foreground shrink-0 font-mono text-xs">
          {megabytes(install.progressBytes)} / {megabytes(install.totalBytes)}
        </span>
      </div>
    {:else if install.state === "failed"}
      <!-- The engine's own sentence. A digest mismatch and a dead connection
           need different responses, and flattening them into "failed" hides
           which one happened. -->
      <p class="text-destructive text-sm">{install.error}</p>
    {:else if install.state === "cancelled"}
      <p class="text-muted-foreground text-sm">
        Stopped at {megabytes(install.progressBytes)}. Installing again resumes
        from there.
      </p>
    {/if}
  {/if}
</div>

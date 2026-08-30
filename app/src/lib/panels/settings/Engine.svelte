<!--
  What the engine found when it started.

  Every value here is reported rather than configured, which is why none of them
  is editable yet. The backend especially: a build that quietly fell back to the
  CPU passes every correctness test while delivering none of the speed, so it is
  stated on screen rather than inferred from how slow things feel.
-->
<script lang="ts">
  import SettingRow from "$lib/ui/SettingRow.svelte";
  import type { SettingsState } from "./state.svelte";

  interface Props {
    /** The panel state this section reads. */
    state: SettingsState;
  }

  let { state }: Props = $props();

  /** Device memory, when the backend reports any. */
  function gigabytes(bytes: number | null): string | null {
    return bytes === null ? null : `${(bytes / 1024 ** 3).toFixed(1)} GB`;
  }
</script>

<section class="flex flex-col gap-2">
  <h2 class="text-muted-foreground text-xs tracking-wide uppercase">Engine</h2>

  <div class="border-border border">
    <SettingRow
      label="Data directory"
      description="Segments, the write-ahead log, models and metadata."
    >
      {#snippet control()}
        <span class="text-muted-foreground selectable font-mono text-xs">
          {state.capabilities?.data_dir ?? "—"}
        </span>
      {/snippet}
    </SettingRow>

    <SettingRow
      label="Compute backend"
      description={state.capabilities?.environment.overridden
        ? "Pinned by TELIVIDB_DEVICE rather than detected."
        : "Chosen by detection when the process started."}
      tag={state.capabilities?.environment.overridden ? "pinned" : undefined}
    >
      {#snippet control()}
        <span class="text-foreground font-mono text-xs">
          {state.capabilities?.environment.backend ?? "—"}
        </span>
        {#if state.capabilities}
          {@const total = gigabytes(state.capabilities.environment.total_bytes)}
          {#if total}
            <span class="text-muted-foreground font-mono text-xs">{total}</span>
          {/if}
        {/if}
      {/snippet}
    </SettingRow>

    <SettingRow
      label="Listen on"
      description="The address the engine serves on, for this machine only."
    >
      {#snippet control()}
        <span class="text-muted-foreground selectable font-mono text-xs">
          {state.capabilities?.address ?? "—"}
        </span>
      {/snippet}
    </SettingRow>

    <SettingRow
      label="Embedding model"
      description="Text can only be stored or searched while one is resident."
    >
      {#snippet control()}
        <span class="text-xs {state.capabilities?.has_model ? 'text-foreground' : 'text-muted-foreground'}">
          {state.capabilities?.has_model ? "loaded" : "none loaded"}
        </span>
      {/snippet}
    </SettingRow>
  </div>
</section>

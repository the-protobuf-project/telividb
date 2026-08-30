<!--
  What the engine found when it started.

  Every value is reported rather than configured, which is why none is editable.
  The backend especially: a build that quietly fell back to the host passes every
  correctness test while delivering none of the speed, so it is stated here
  rather than inferred from how slow things feel.
-->
<script lang="ts">
  import { SettingGroup, SettingRow, Tag } from "$lib/ui";
  import type { SettingsState } from "./state.svelte";

  interface Props {
    /** The panel state this section reads. */
    state: SettingsState;
  }

  let { state }: Props = $props();

  let env = $derived(state.capabilities?.environment ?? null);

  /** Bytes as gigabytes, or a dash when nothing was reported. */
  function gigabytes(bytes: number): string {
    return bytes === 0 ? "—" : `${(bytes / 1024 ** 3).toFixed(1)} GB`;
  }
</script>

<SettingGroup>
  <SettingRow
    label="Data directory"
    description="Segments, the write-ahead log, models and metadata."
  >
    {#snippet control()}
      <span class="mono faint selectable" style="font-size: 0.75rem">
        {state.capabilities?.data_dir ?? "—"}
      </span>
    {/snippet}
  </SettingRow>

  <SettingRow
    label="Compute backend"
    description={env?.budgetSource === "configured"
      ? "Pinned by TELIVIDB_DEVICE rather than detected."
      : "Chosen by detection when the process started."}
    tag={env?.budgetSource === "configured" ? "pinned" : undefined}
    tone="amber"
  >
    {#snippet control()}
      <span class="mono" style="font-size: 0.75rem">{env?.backend ?? "—"}</span>
      <span class="mono faint" style="font-size: 0.75rem">
        {env ? gigabytes(env.budgetLimitBytes) : ""}
      </span>
    {/snippet}
  </SettingRow>

  <SettingRow
    label="Listen on"
    description="The address the engine serves on, for this machine only."
  >
    {#snippet control()}
      <span class="mono faint selectable" style="font-size: 0.75rem">
        {state.capabilities?.address ?? "—"}
      </span>
    {/snippet}
  </SettingRow>

  <SettingRow
    label="Embedding model"
    description="Text can only be stored or searched while one is resident."
  >
    {#snippet control()}
      <Tag tone={state.capabilities?.has_model ? "green" : "plain"}>
        {state.capabilities?.has_model ? "loaded" : "none loaded"}
      </Tag>
    {/snippet}
  </SettingRow>
</SettingGroup>

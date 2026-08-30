<!--
  What the engine found when it started.

  Every value is reported rather than configured, which is why none is editable.
  The backend especially: a build that quietly fell back to the host passes every
  correctness test while delivering none of the speed, so it is stated here
  rather than inferred from how slow things feel.
-->
<script lang="ts">
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

<div>
  <div class="panel-label">Engine</div>
  <div class="set-group" style="margin-top: 0.625rem">
    <div class="set-row">
      <div class="txt">
        <b>Data directory</b>
        <span>Segments, the write-ahead log, models and metadata.</span>
      </div>
      <div class="ctl">
        <span class="mono faint selectable" style="font-size: 0.75rem">
          {state.capabilities?.data_dir ?? "—"}
        </span>
      </div>
    </div>

    <div class="set-row">
      <div class="txt">
        <b>Compute backend</b>
        <span>
          {env?.budgetSource === "configured"
            ? "Pinned by TELIVIDB_DEVICE rather than detected."
            : "Chosen by detection when the process started."}
        </span>
      </div>
      <div class="ctl" style="display: flex; gap: 0.5rem; align-items: center">
        <span class="mono" style="font-size: 0.75rem">{env?.backend ?? "—"}</span>
        <span class="faint mono" style="font-size: 0.75rem">
          {env ? gigabytes(env.budgetLimitBytes) : ""}
        </span>
      </div>
    </div>

    <div class="set-row">
      <div class="txt">
        <b>Listen on</b>
        <span>The address the engine serves on, for this machine only.</span>
      </div>
      <div class="ctl mono faint selectable" style="font-size: 0.75rem">
        {state.capabilities?.address ?? "—"}
      </div>
    </div>

    <div class="set-row">
      <div class="txt">
        <b>Embedding model</b>
        <span>Text can only be stored or searched while one is resident.</span>
      </div>
      <div class="ctl">
        <span class="tag" class:green={state.capabilities?.has_model}>
          {state.capabilities?.has_model ? "loaded" : "none loaded"}
        </span>
      </div>
    </div>
  </div>
</div>

<!--
  What the engine is running on, as tiles.

  Served rather than detected in this window: the desktop build could look at its
  own process only because it links the engine, and a browser talking to a Linux
  daemon cannot. Both now read the same answer.
-->
<script lang="ts">
  import type { MetricsState } from "./state.svelte";

  interface Props {
    /** The panel state this section reads. */
    state: MetricsState;
  }

  let { state }: Props = $props();

  let env = $derived(state.capabilities?.environment ?? null);

  /** Bytes as gigabytes, or a dash when nothing was reported. */
  function gigabytes(bytes: number): string {
    return bytes === 0 ? "—" : (bytes / 1024 ** 3).toFixed(1);
  }
</script>

<div class="tiles">
  <div class="tile">
    <span class="k">Backend</span>
    <span class="v">{env?.backend ?? "—"}</span>
    <!-- A host fallback is not a fault, but it is the thing worth noticing: it
         passes every correctness test while delivering none of the speed. -->
    <span class="n">
      {env ? (env.backend === "cpu" ? "host — no accelerator" : "accelerated") : ""}
    </span>
  </div>

  <div class="tile">
    <span class="k">Device</span>
    <span class="v" style="font-size: 0.9375rem" title={env?.device}>
      {env?.device ?? "—"}
    </span>
    <span class="n">what ggml opened</span>
  </div>

  <div class="tile">
    <span class="k">Memory budget</span>
    <span class="v">
      {env ? gigabytes(env.budgetLimitBytes) : "—"}<small>GB</small>
    </span>
    <!-- An estimate on a discrete card overshoots, so which kind of number this
         is matters as much as the number. -->
    <span class="n">{env?.budgetSource ?? ""}</span>
  </div>

  <div class="tile">
    <span class="k">Resident</span>
    <span class="v">
      {env ? gigabytes(env.budgetUsedBytes) : "—"}<small>GB</small>
    </span>
    <span class="n">held by indexes and models</span>
  </div>

  <div class="tile">
    <span class="k">Engine</span>
    <span class="v" style="font-size: 1rem">{env?.version ?? "—"}</span>
    <span class="n">build that answered</span>
  </div>
</div>

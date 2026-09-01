<!--
  What the engine is running on, as tiles.

  Served rather than detected in this window: the desktop build could look at its
  own process only because it links the engine, and a browser talking to a Linux
  daemon cannot. Both now read the same answer.
-->
<script lang="ts">
  import { Tile, Tiles } from "$lib/ui";
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

<Tiles>
  <Tile
    label="Backend"
    value={env?.backend ?? "—"}
    note={env ? (env.backend === "cpu" ? "host — no accelerator" : "accelerated") : ""}
  />
  <Tile label="Device" value={env?.device ?? "—"} note="what ggml opened" compact />
  <Tile
    label="Memory budget"
    value={env ? gigabytes(env.budgetLimitBytes) : "—"}
    unit={env && env.budgetLimitBytes > 0 ? "GB" : undefined}
    note={env?.budgetSource ?? ""}
  />
  <Tile
    label="Resident"
    value={env ? gigabytes(env.budgetUsedBytes) : "—"}
    unit={env && env.budgetUsedBytes > 0 ? "GB" : undefined}
    note="held by indexes and models"
  />
  <Tile label="Engine" value={env?.version ?? "—"} note="build that answered" compact />
</Tiles>

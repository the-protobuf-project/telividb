<!--
  What this is, and who made it.

  The licence is the workspace's own rather than a hopeful second one: every
  crate inherits `license.workspace`, the client SDK included, so naming a more
  permissive licence for the SDK would describe a plan rather than the build
  someone is running.
-->
<script lang="ts">
  import { Kv, SettingGroup } from "$lib/ui";
  import type { SettingsState } from "./state.svelte";

  interface Props {
    /** The panel state, for the runtime line. */
    state: SettingsState;
  }

  let { state }: Props = $props();

  let facts = $derived([
    ["Author", "Srikanth Kandarp"],
    ["Project", "the-protobuf-project"],
    ["Licence", "AGPL-3.0-or-later"],
    ["Runtime", `ggml · ${state.capabilities?.environment.backend ?? "—"}`],
    ["Storage", "redb · mmap segments"],
    ["Built with", "Rust · Tauri · Svelte"],
  ] as const);
</script>

<SettingGroup>
  <div class="set-row" style="flex-direction: column; align-items: flex-start; gap: calc(var(--u) * 3)">
    <div style="display: flex; align-items: baseline; gap: calc(var(--u) * 2)">
      <span class="wordmark" style="font-size: 1.0625rem">telivi<span>db</span></span>
      <span class="mono faint" style="font-size: 0.75rem">0.1.0</span>
    </div>

    <p class="lede" style="margin: 0">
      A single-node vector and graph database with a window on top. Bring your own
      embedding model, bring your own search algorithm — the engine runs in this
      process, and retrieval never leaves this machine.
    </p>

    <div style="display: flex; flex-direction: column; gap: var(--u); width: 100%; max-width: 26rem">
      {#each facts as [term, value] (term)}
        <Kv label={term} {value} />
      {/each}
    </div>
  </div>
</SettingGroup>

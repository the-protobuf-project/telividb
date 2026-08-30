<!--
  What this is, and who made it.

  The licence is the workspace's own rather than a hopeful second one: every
  crate inherits `license.workspace`, the client SDK included, so naming a more
  permissive licence for the SDK would describe a plan rather than the build
  someone is running.
-->
<script lang="ts">
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

<div>
  <div class="set-group">
    <div class="set-row" style="flex-direction: column; align-items: flex-start; gap: 0.75rem">
      <div class="about">
        <div style="display: flex; align-items: baseline; gap: 0.5rem">
          <span class="wordmark" style="font-size: 1.0625rem">telivi<span>db</span></span>
          <span class="mono faint" style="font-size: 0.75rem">0.1.0</span>
        </div>
        <p class="lede" style="margin: 0">
          A single-node vector and graph database with a window on top. Bring your
          own embedding model, bring your own search algorithm — the engine runs
          in this process, and retrieval never leaves this machine.
        </p>
        <dl>
          {#each facts as [term, value] (term)}
            <dt>{term}</dt>
            <dd class="selectable">{value}</dd>
          {/each}
        </dl>
      </div>
    </div>
  </div>
</div>

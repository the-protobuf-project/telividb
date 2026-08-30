<!--
  What this is, and who made it.

  The licence is the workspace's own rather than a hopeful second one: every
  crate here inherits `license.workspace`, the client SDK included, so naming a
  more permissive licence for the SDK would be describing a plan rather than the
  build someone is running.
-->
<script lang="ts">
  import type { SettingsState } from "./state.svelte";

  interface Props {
    /** The panel state, for the runtime line. */
    state: SettingsState;
  }

  let { state }: Props = $props();

  /** Facts, in the order a reader wants them. */
  let facts = $derived([
    ["Author", "Srikanth Kandarp"],
    ["Project", "the-protobuf-project"],
    ["Licence", "AGPL-3.0-or-later"],
    ["Runtime", `ggml · ${state.capabilities?.environment.backend ?? "—"}`],
    ["Storage", "redb · mmap segments"],
    ["Built with", "Rust · Tauri · Svelte"],
  ] as const);
</script>

<section class="flex flex-col gap-2">
  <h2 class="text-muted-foreground text-xs tracking-wide uppercase">About</h2>

  <div class="border-border flex flex-col gap-4 border p-4">
    <div class="flex items-baseline gap-2">
      <span class="text-foreground text-lg font-medium tracking-tight">
        telivi<span class="text-muted-foreground">db</span>
      </span>
      <span class="text-muted-foreground font-mono text-xs">0.1.0</span>
    </div>

    <p class="text-muted-foreground max-w-prose text-sm">
      A single-node vector and graph database with a window on top. Bring your own
      embedding model, bring your own search algorithm — the engine runs in this
      process, and retrieval never leaves this machine.
    </p>

    <dl class="grid grid-cols-[8rem_1fr] gap-x-4 gap-y-1.5 text-xs">
      {#each facts as [term, value] (term)}
        <dt class="text-muted-foreground">{term}</dt>
        <dd class="text-foreground selectable">{value}</dd>
      {/each}
    </dl>
  </div>
</section>

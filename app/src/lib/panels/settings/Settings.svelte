<!--
  Settings.

  The design's page shell: `.page` is a two-row grid, `.page-top` holds the
  heading and does not move, `.page-scroll` is the only thing that scrolls, and
  `.page-inner` centres both on the same 58rem column so the heading sits above
  its own content rather than above the window.
-->
<script lang="ts">
  import { client } from "$lib/api";
  import { SettingsState } from "./state.svelte";
  import Engine from "./Engine.svelte";
  import Providers from "./Providers.svelte";
  import Privacy from "./Privacy.svelte";
  import About from "./About.svelte";

  const state = new SettingsState(client);

  // Read once on mount. Nothing polls: these change when the engine restarts,
  // and the panel is rebuilt when it does.
  void state.load();
</script>

<div class="page">
  <div class="page-top">
    <div class="page-inner">
      <div class="page-head"><h1>Settings</h1></div>
    </div>
  </div>

  <div class="page-scroll">
    <div class="page-inner">
      {#if state.error}
        <p class="selectable" style="color: var(--red-text)">{state.error}</p>
      {/if}

      <Engine {state} />
      <Providers {state} />
      <Privacy />
      <About {state} />
    </div>
  </div>
</div>

<!--
  Settings, split left and right rather than stacked.

  Sections list on the left, the chosen one fills the right. That is the shape
  because the alternative is one tall column that has to be scrolled to be read
  — and a window that scrolls has lost the property this whole frame is built
  around: the header, the dock and the controls stay where the hand left them.

  Each side owns its own overflow. A section long enough to need it scrolls
  inside its own pane; the frame never does.
-->
<script lang="ts">
  import { client } from "$lib/api";
  import { Notice, TreeNode } from "$lib/ui";
  import { SECTIONS, SettingsState } from "./state.svelte";
  import Engine from "./Engine.svelte";
  import Providers from "./Providers.svelte";
  import Privacy from "./Privacy.svelte";
  import About from "./About.svelte";

  const state = new SettingsState(client);

  // Read once on mount. Nothing polls: these change when the engine restarts,
  // and the panel is rebuilt when it does.
  void state.load();

  let current = $derived(SECTIONS.find((s) => s.id === state.section));
</script>

<div class="view two">
  <nav class="rail">
    <div>
      <div class="rail-label">Settings</div>
      <div class="tree">
        {#each SECTIONS as section (section.id)}
          <TreeNode
            name={section.label}
            kind="org"
            count={section.summary}
            current={state.section === section.id}
            onclick={() => (state.section = section.id)}
          />
        {/each}
      </div>
    </div>
  </nav>

  <div class="page">
    <div class="page-top">
      <div class="page-inner">
        <div class="page-head"><h1>{current?.label ?? "Settings"}</h1></div>
      </div>
    </div>

    <div class="page-body">
      <div class="page-inner">
        {#if state.error}
          <Notice tone="error">{state.error}</Notice>
        {/if}

        {#if state.section === "engine"}
          <Engine {state} />
        {:else if state.section === "answering"}
          <Providers {state} />
        {:else if state.section === "privacy"}
          <Privacy />
        {:else}
          <About {state} />
        {/if}
      </div>
    </div>
  </div>
</div>

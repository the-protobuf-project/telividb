<!--
  Settings.

  A fixed heading and one scrolling body, which is the rule the whole window
  follows: the frame never scrolls, a pane inside it does. The heading stays put
  so the panel's identity does not slide away under a long list.
-->
<script lang="ts">
  import { client } from "$lib/api";
  import { SettingsState } from "./state.svelte";
  import Engine from "./Engine.svelte";
  import Providers from "./Providers.svelte";
  import Privacy from "./Privacy.svelte";
  import About from "./About.svelte";

  const state = new SettingsState(client);

  // Read once on mount. Nothing here polls: these values change when the engine
  // restarts, and the panel is rebuilt when it does.
  void state.load();
</script>

<div class="flex h-full min-h-0 flex-col">
  <div class="border-border shrink-0 border-b px-6 py-4">
    <h1 class="text-foreground text-lg font-medium">Settings</h1>
  </div>

  <div class="min-h-0 flex-1 overflow-y-auto px-6 py-5">
    <div class="mx-auto flex max-w-3xl flex-col gap-8">
      {#if state.error}
        <p class="text-destructive selectable text-sm">{state.error}</p>
      {/if}

      <Engine {state} />
      <Providers {state} />
      <Privacy />
      <About {state} />
    </div>
  </div>
</div>

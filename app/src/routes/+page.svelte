<!--
  The shell.

  Which panel is showing lives here rather than in a store: it is one string
  read by two components that are both on this page, and a store would add a
  module to reach state that never leaves it.
-->
<script lang="ts">
  import { client } from "$lib/api";
  import Sidebar from "$lib/ui/Sidebar.svelte";
  import StatusBar from "$lib/ui/StatusBar.svelte";
  import Placeholder from "$lib/ui/Placeholder.svelte";
  import Search from "$lib/panels/search/Search.svelte";

  let active = $state("search");
  let address = $state<string | null>(null);

  // The engine is already running by the time the window opens — the app
  // refuses to start otherwise — so this only fails if the bridge itself is
  // broken, which is not something to render an error banner for.
  $effect(() => {
    client
      .engineAddress()
      .then((a) => (address = a))
      .catch(() => (address = null));
  });

  const waiting: Record<string, string> = {
    points: "Not built yet. Points.List and Get are served and ready.",
    schema:
      "Not built yet. Reads vector_fields from Collections.Get, which is served.",
    graph:
      "The Graph service is not served yet. The traversal engine is built, but it returns names without edges or paths, so there is nothing to draw.",
    system:
      "The SystemInfo service is not served yet. Until it is, the backend the engine selected is not readable from here.",
  };
</script>

<StatusBar {address} />

<div class="flex min-h-0 flex-1">
  <Sidebar bind:active />

  <main class="min-w-0 flex-1">
    {#if active === "search"}
      <Search />
    {:else}
      <Placeholder
        title={active.charAt(0).toUpperCase() + active.slice(1)}
        waiting={waiting[active] ?? "Not built yet."}
      />
    {/if}
  </main>
</div>

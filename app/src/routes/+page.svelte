<!--
  The shell.

  Which panel is showing, and which collection the panels act on, live here:
  both are read by more than one child and neither leaves this page, so a store
  would be a module to reach state that never travels.
-->
<script lang="ts">
  import { client } from "$lib/api";
  import Sidebar from "$lib/ui/Sidebar.svelte";
  import StatusBar from "$lib/ui/StatusBar.svelte";
  import Placeholder from "$lib/ui/Placeholder.svelte";
  import Search from "$lib/panels/search/Search.svelte";
  import Collections from "$lib/panels/collections/Collections.svelte";
  import Points from "$lib/panels/points/Points.svelte";
  import { Intro } from "$lib/motion/intro";

  let active = $state("collections");
  let address = $state<string | null>(null);
  // Assumed false until the engine says otherwise: offering an import that
  // will be refused is worse than withholding one that would have worked.
  let canEmbed = $state(false);
  let collection = $state("");

  let markRef = $state<HTMLElement | null>(null);
  let barRef = $state<HTMLElement | null>(null);
  let sidebarRef = $state<HTMLElement | null>(null);
  let panelRef = $state<HTMLElement | null>(null);

  const intro = new Intro();

  // The engine is already running by the time the window opens — the app
  // refuses to start otherwise — so this only fails if the bridge itself is
  // broken, which is not something to render a banner for.
  $effect(() => {
    client
      .capabilities()
      .then((c) => {
        address = c.address;
        canEmbed = c.has_model;
      })
      .catch(() => (address = null));
  });

  // Every part has to be mounted before the sequence can move it.
  $effect(() => {
    if (markRef && barRef && sidebarRef && panelRef) {
      intro.play({
        mark: markRef,
        bar: barRef,
        sidebar: sidebarRef,
        panel: panelRef,
      });
    }
  });

  const waiting: Record<string, string> = {
    schema:
      "Not built yet. Reads vector_fields from Collections.Get, which is served.",
    graph:
      "The Graph service is not served yet. The traversal engine is built, but it returns names without edges or paths, so there is nothing to draw.",
    system:
      "The SystemInfo service is not served yet. Until it is, the backend the engine selected is not readable from here.",
  };
</script>

<StatusBar {address} bind:ref={barRef} bind:markRef />

<div class="flex min-h-0 flex-1">
  <Sidebar bind:active bind:ref={sidebarRef} />

  <main bind:this={panelRef} class="min-w-0 flex-1">
    {#if active === "search"}
      <Search />
    {:else if active === "collections"}
      <Collections
        oncreated={(created) => {
          collection = created;
          active = "points";
        }}
      />
    {:else if active === "points"}
      <Points {collection} {canEmbed} />
    {:else}
      <Placeholder
        title={active.charAt(0).toUpperCase() + active.slice(1)}
        waiting={waiting[active] ?? "Not built yet."}
      />
    {/if}
  </main>
</div>

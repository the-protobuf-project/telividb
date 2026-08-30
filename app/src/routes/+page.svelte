<!--
  The shell.

  Which panel is showing, and which collection the panels act on, live here:
  both are read by more than one child and neither leaves this page, so a store
  would be a module to reach state that never travels.
-->
<script lang="ts">
  import { EngineStatus } from "$lib/api/status.svelte";
  import Dock from "$lib/ui/Dock.svelte";
  import TopNav from "$lib/ui/TopNav.svelte";
  import Placeholder from "$lib/ui/Placeholder.svelte";
  import Ask from "$lib/panels/ask/Ask.svelte";
  import Search from "$lib/panels/search/Search.svelte";
  import Collections from "$lib/panels/collections/Collections.svelte";
  import Points from "$lib/panels/points/Points.svelte";
  import Models from "$lib/panels/models/Models.svelte";
  import Settings from "$lib/panels/settings/Settings.svelte";
  import Workspace from "$lib/panels/workspace/Workspace.svelte";
  import Metrics from "$lib/panels/metrics/Metrics.svelte";
  import Graph from "$lib/panels/graph/Graph.svelte";
  import People from "$lib/panels/people/People.svelte";
  import { Intro } from "$lib/motion/intro";
  import { enter } from "$lib/motion/motion";
  import Onboarding from "$lib/onboarding/Onboarding.svelte";
  import { OnboardingState } from "$lib/onboarding/state.svelte";

  // Shown until this machine has been through it once. Read synchronously so
  // the app does not flash the shell before deciding.
  let onboarding = $state(!OnboardingState.seen());
  // Ask, because it is the panel that demonstrates what the engine does with
  // nothing else in the way — no schema to fill in, no file to map.
  let active = $state("workspace");
  // Kept because the capabilities poll writes it and a null means the bridge
  // is unreachable; the value itself is shown in Settings, not up here.
  let collection = $state("");

  /** Everything the shell shows about the engine, and the polling behind it. */
  const engine = new EngineStatus();

  /** Re-read what the engine can do. Handed to panels that change it. */
  function refreshCapabilities() {
    engine.refresh();
  }


  let markRef = $state<HTMLElement | null>(null);
  let barRef = $state<HTMLElement | null>(null);
  let dockRef = $state<HTMLElement | null>(null);
  /**
   * Whether the rail shows names.
   *
   * Remembered across launches: it is a preference about how this person reads
   * the app, and re-collapsing it every morning would be the window forgetting
   * something it was told.
   */
  let dockOpen = $state(
    typeof localStorage !== "undefined" && localStorage.getItem("dock") === "open",
  );

  $effect(() => {
    localStorage.setItem("dock", dockOpen ? "open" : "closed");
  });

  const intro = new Intro();

  // The panel animates when the dock switches, not on first paint — the launch
  // sequence owns that moment, and two entrances on the same element would
  // interpolate from wherever the first one had reached.
  let entered = $state(false);
  $effect(() => {
    // Read so the effect re-runs on a switch; the value itself is not needed.
    void active;
    if (!entered) {
      entered = true;
      return;
    }
    const panel = document.querySelector<HTMLElement>(".app > .view");
    if (panel) enter(panel);
  });

  // The engine is already running by the time the window opens — the app
  // refuses to start otherwise — so this only fails if the bridge itself is
  // broken, which is not something to render a banner for.
  /**
   * Re-read what the engine can do.
   *
   * Called on open and again whenever a model becomes resident: installing one
   * changes whether text can be embedded, and nothing else would tell the
   * window.
   */

  $effect(() => {
    refreshCapabilities();

    // The engine binds before it loads a model, so the window can open a good
    // twenty seconds before text is possible. Without this it would sit at "no
    // model" until something else happened to ask.
    if (engine.canEmbed) return;
    const timer = setInterval(() => {
      if (engine.canEmbed) {
        clearInterval(timer);
        return;
      }
      refreshCapabilities();
      engine.refreshOrganizations();
    }, 2000);
    return () => clearInterval(timer);
  });

  // Every part has to be mounted before the sequence can move it, and none of
  // them exist while onboarding is on screen.
  $effect(() => {
    if (!onboarding && markRef && barRef && dockRef) {
      // Queried rather than bound: the panel is whichever `.view` the dock is
      // showing, and it is a direct child of `.app` with nothing to bind to.
      const panel = document.querySelector<HTMLElement>(".app > .view");
      if (panel) {
        intro.play({ mark: markRef, bar: barRef, sidebar: dockRef, panel });
      }
    }
  });

  const waiting: Record<string, string> = {
    schema:
      "Not built yet. Reads vector_fields from Collections.Get, which is served.",
    graph:
      "The Graph service is not served yet. The traversal engine is built, but it returns names without edges or paths, so there is nothing to draw.",
    people:
      "The Identity service is not served yet — users, groups and role bindings have protos and no store behind them.",
  };
</script>

{#if onboarding}
  <Onboarding
    ondone={() => {
      onboarding = false;
      engine.refreshOrganizations();
      engine.refresh();
    }}
  />
{:else}
  <TopNav
    organization={engine.organization}
    model={engine.model}
    backend={engine.capabilities?.environment.backend ?? null}
    bind:ref={barRef}
    bind:markRef
  />

  <!-- `.app` is a `3.25rem 1fr` grid under a `--nav-h` bar: the dock, then the
       view itself as the second column.
       
       No wrapper between them. There was one — a `<main class="main">` — and it
       is what made every panel look cut off: `.main` is `grid-template-rows:
       auto 1fr auto` because in the design it is the *workspace view's centre
       column*, holding the space head, the thread and the composer. Given a
       single child it put it in the first `auto` row and clipped the rest with
       its own `overflow: hidden`. -->
  <div class="app" data-dock={dockOpen ? "open" : "closed"}>
    <Dock bind:active bind:open={dockOpen} bind:ref={dockRef} />

      {#if active === "workspace"}
        <Workspace canEmbed={engine.canEmbed} />
      {:else if active === "data"}
        <Points {collection} canEmbed={engine.canEmbed} />
      {:else if active === "models"}
        <Models oninstalled={refreshCapabilities} />
      {:else if active === "graph"}
        <Graph />
      {:else if active === "people"}
        <People />
      {:else if active === "metrics"}
        <Metrics />
      {:else if active === "settings"}
        <Settings />
      {:else if active === "ask"}
        <Ask canEmbed={engine.canEmbed} />
      {:else if active === "search"}
        <Search />
      {:else if active === "collections"}
        <Collections
          oncreated={(created) => {
            collection = created;
            active = "data";
          }}
        />
      {:else}
        <Placeholder
          title={active.charAt(0).toUpperCase() + active.slice(1)}
          waiting={waiting[active] ?? "Not built yet."}
        />
      {/if}
  </div>
{/if}

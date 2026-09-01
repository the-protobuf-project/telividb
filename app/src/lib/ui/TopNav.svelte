<!--
  The window's top bar: wordmark, where you are, and what is loaded.

  The two chips on the right are the design's answer to a question nothing else
  on screen answers — which model is in memory, and which backend the engine
  actually selected. A build that quietly fell back to the host looks identical
  from every other angle.
-->
<script lang="ts">
  interface Props {
    /** The organization in the breadcrumb, or null before one is chosen. */
    organization?: string | null;
    /** The project in the breadcrumb. */
    project?: string | null;
    /** The resident embedding model's name, when one is loaded. */
    model?: string | null;
    /** The compute backend the engine reported. */
    backend?: string | null;
    /** The bar itself, for the launch sequence to move. */
    ref?: HTMLElement | null;
    /** The wordmark, which the launch sequence resolves from. */
    markRef?: HTMLElement | null;
  }

  let {
    organization = null,
    project = null,
    model = null,
    backend = null,
    ref = $bindable(null),
    markRef = $bindable(null),
  }: Props = $props();
</script>

<header class="nav" bind:this={ref}>
  <div class="wordmark" bind:this={markRef}>telivi<span>db</span></div>

  <!-- Text, not controls, and the chevrons are gone with them.

       These were two `<button>`s with a disclosure chevron, no handler and no
       `aria-haspopup` — so they were the first two tab stops in the window,
       they announced as actionable, they promised a menu, and they did nothing.
       An em-dash was their whole accessible name. They become buttons again on
       the day something opens; until then the bar states where you are. -->
  <div class="crumb">
    <span class="sep">/</span>
    <span class="crumb-at">{organization ?? "—"}</span>
    <span class="sep">/</span>
    <b class="crumb-at">{project ?? "—"}</b>
  </div>

  <!-- What each chip means is carried in text as well as in `title`. A `title`
       on a non-focusable div is reachable by hovering mouse only, so the
       explanation was the one part of these chips a keyboard never reached. -->
  <div class="nav-right">
    <div class="chip" title="The model held in memory right now">
      <!-- Live only when something is actually resident. A green dot beside a
           dash would say the opposite of what the dash says. -->
      <span class="dot" class:live={!!model}></span>
      <span class="sr-only">Model in memory:</span>
      <span>{model ?? "no model"}</span>
    </div>
    <div class="chip" title="The compute backend this engine selected">
      <span class="dot" class:live={!!backend && backend !== "cpu"}></span>
      <span class="sr-only">Compute backend:</span>
      {backend ?? "unknown"}
    </div>
  </div>
</header>

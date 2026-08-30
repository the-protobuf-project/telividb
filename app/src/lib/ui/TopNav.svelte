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

  <div class="crumb">
    <span class="sep">/</span>
    <button class="crumb-btn" type="button">
      <span>{organization ?? "—"}</span>
      <svg width="9" height="9" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.3">
        <path d="M3 5l3 3 3-3" />
      </svg>
    </button>
    <span class="sep">/</span>
    <button class="crumb-btn" type="button">
      <b>{project ?? "—"}</b>
      <svg width="9" height="9" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.3">
        <path d="M3 5l3 3 3-3" />
      </svg>
    </button>
  </div>

  <div class="nav-right">
    <div class="chip" title="The model held in memory right now">
      <!-- Live only when something is actually resident. A green dot beside a
           dash would say the opposite of what the dash says. -->
      <span class="dot" class:live={!!model}></span>
      <span>{model ?? "no model"}</span>
    </div>
    <div class="chip" title="The compute backend this engine selected">
      <span class="dot" class:live={!!backend && backend !== "cpu"}></span>
      {backend ?? "unknown"}
    </div>
  </div>
</header>

<!--
  The navigation rail, collapsed and expanded.

  Collapsed is the design's own icon rail — compact, but cryptic until the six
  glyphs are learned. Expanded names them. Click the control at the foot to move
  between the two; the app remembers which you chose.
-->
<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import Dock from "./Dock.svelte";

  const { Story } = defineMeta({ title: "Elements/Dock", parameters: { layout: "fullscreen" } });

  let active = $state("workspace");
  let open = $state(true);
  let activeClosed = $state("metrics");
  let closed = $state(false);
</script>

<!-- Both states at once, which is the comparison worth making — and the
     transition is on the grid column, so it only reads correctly inside one. -->
<Story name="Collapsed and expanded">
  <div style="display: flex; height: 100vh">
    <div class="app" data-dock={closed ? "open" : "closed"} style="height: 100%">
      <Dock bind:active={activeClosed} bind:open={closed} />
      <div style="padding: var(--gutter)">
        <p class="hint">Collapsed — hover a glyph for its name.</p>
      </div>
    </div>

    <div class="app" data-dock={open ? "open" : "closed"} style="height: 100%; border-left: 1px solid var(--rule)">
      <Dock bind:active bind:open />
      <div style="padding: var(--gutter)">
        <p class="hint">Expanded — click the control at the foot to collapse.</p>
      </div>
    </div>
  </div>
</Story>

<!--
  The window: a status bar, a sidebar, and one panel.

  The stylesheet is imported here so it applies to every route. There is exactly
  one, and it holds the tokens rather than the components — a component that
  names a colour directly would have to be edited twice when the palette moves.
-->
<script lang="ts">
  import "../app.css";

  let { children } = $props();
</script>

<div class="flex h-full flex-col">
  {@render children()}
</div>

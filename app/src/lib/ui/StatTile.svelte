<!--
  Kept as the name earlier panels import; `Tile` is the component.
  
  A re-export rather than a second implementation, so there is one tile in this
  design system and not two that drift.
-->
<script lang="ts">
  import Tile from "./Tile.svelte";

  interface Props {
    /** What is being measured. */
    label: string;
    /** The figure. */
    value: string;
    /** A unit shown small beside it. */
    unit?: string;
    /** What the figure means, or where it came from. */
    note?: string;
  }

  let { label, value, unit, note }: Props = $props();
</script>

<Tile {label} {value} {unit} {note} />

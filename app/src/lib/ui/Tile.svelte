<!--
  One measurement: a label, a large figure, and what the figure means.

  The note underneath is not optional in practice — "28.8 GB" is a different
  claim depending on whether it was measured or estimated, and the tile is where
  that distinction has room to be stated.
-->
<script lang="ts">
  interface Props {
    /** What is being measured. */
    label: string;
    /** The figure. */
    value: string;
    /** A unit shown small beside it. */
    unit?: string;
    /** What the figure means, or where it came from. */
    note?: string;
    /** Smaller type, for a value that is a name rather than a number. */
    compact?: boolean;
  }

  let { label, value, unit, note, compact = false }: Props = $props();
</script>

<div class="tile">
  <span class="k">{label}</span>
  <span class="v" style={compact ? "font-size: 1rem" : ""}>
    {value}{#if unit}<small>{unit}</small>{/if}
  </span>
  {#if note}<span class="n">{note}</span>{/if}
</div>

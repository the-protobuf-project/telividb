<!--
  Tiles as they actually appear: in a row, sharing a hairline.
-->
<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import StatTile from "./StatTile.svelte";

  const { Story } = defineMeta({ title: "Readouts/StatTile", component: StatTile });
</script>

<Story name="A row">
  <!-- The 1px gap over a rule colour is what draws the dividers, so the tiles
       share a hairline instead of each carrying its own border. -->
  <div class="border-border bg-border grid grid-cols-[repeat(auto-fit,minmax(9rem,1fr))] gap-px border">
    <StatTile label="Embed p50" value="0.42" unit="ms" note="18 calls" />
    <StatTile label="Search p50" value="0.08" unit="ms" note="over 24 points" />
    <StatTile label="Resident" value="609.5" unit="MiB" note="model on device" />
    <StatTile label="Backend" value="metal" note="Apple M3 Max" />
  </div>
</Story>

<Story name="Nothing recorded yet">
  <div class="border-border bg-border grid grid-cols-2 gap-px border">
    <StatTile label="Embed p50" value="—" note="no calls yet" />
    <StatTile label="Points" value="0" />
  </div>
</Story>

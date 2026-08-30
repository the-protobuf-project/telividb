<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import Tile from "./Tile.svelte";
  import Tiles from "./Tiles.svelte";

  const { Story } = defineMeta({ title: "Elements/Tile" });
</script>

<!-- One pixel of gap over a ruled ground, so they read as one panel. -->
<Story name="Instrument panel">
  <Tiles>
    <Tile label="Backend" value="metal" note="accelerated" />
    <Tile label="Budget" value="28.8" unit="GB" note="measured" />
    <Tile label="Resident" value="0.6" unit="GB" note="indexes and models" />
    <Tile label="Engine" value="0.1.0" note="build that answered" compact />
  </Tiles>
</Story>

<!-- What it shows before anything has been measured. -->
<Story name="Nothing yet">
  <Tiles>
    <Tile label="Backend" value="—" />
    <Tile label="Budget" value="—" note="not reported" />
  </Tiles>
</Story>

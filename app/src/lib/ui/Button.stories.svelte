<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import Button from "./Button.svelte";

  const { Story } = defineMeta({ title: "Elements/Button" });
</script>

<!-- All four at once, which is how they appear beside each other in a card foot. -->
<Story name="Every variant">
  <div style="display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap">
    <Button>Primary</Button>
    <Button size="sm">Primary small</Button>
    <Button variant="ghost">Ghost</Button>
    <Button variant="ghost" size="sm">Ghost small</Button>
    <Button disabled>Disabled</Button>
  </div>
</Story>

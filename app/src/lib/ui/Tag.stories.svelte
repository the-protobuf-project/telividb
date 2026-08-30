<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import Tag from "./Tag.svelte";

  const { Story } = defineMeta({ title: "Elements/Tag" });
</script>

<!-- Each tone asserts something; the labels here are the ones the app uses. -->
<Story name="Every tone">
  <div style="display: flex; gap: 0.5rem; align-items: center">
    <Tag>installed</Tag>
    <Tag tone="green">in memory</Tag>
    <Tag tone="amber">sample data</Tag>
    <Tag tone="blue">recommended</Tag>
  </div>
</Story>

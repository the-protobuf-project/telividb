<!--
  People, on sample data.

  The screen most at risk of lying: a list of users and roles reads as a working
  permission system, and this one grants nothing. The correction sits above the
  list rather than in a footnote.
-->
<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import People from "./People.svelte";

  const { Story } = defineMeta({ title: "People", parameters: { layout: "fullscreen" } });
</script>

<Story name="Groups and grants">
  <div style="height: 100vh">
    <People />
  </div>
</Story>

<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import TreeNode from "./TreeNode.svelte";
  import Lock from "./Lock.svelte";

  const { Story } = defineMeta({ title: "Elements/Tree node" });
  let open = $state("notes");
</script>

<!--
  Depth is the only thing separating the kinds — and a space sits at the same
  depth as a project, because it is a sibling that references projects rather
  than a child of one.
-->
<Story name="A structure rail">
  <div class="tree" style="border: 1px solid var(--rule); max-width: 16rem">
    <TreeNode name="Acme Research" kind="org" count="2p · 3s" />
    <TreeNode name="Retrieval" kind="project" count="retrieval" />
    <TreeNode name="Notes" kind="space" protection="none" current={open === "notes"} onclick={() => (open = "notes")} />
    <TreeNode name="Journal" kind="space" protection="vault" current={open === "journal"} onclick={() => (open = "journal")} />
    <TreeNode name="Board" kind="space" protection="sealed" current={open === "board"} onclick={() => (open = "board")} />
    <TreeNode name="Archived" kind="space" protection="private" muted />
  </div>
</Story>

<!-- Four protections, three glyphs, and deliberately unequal promises. -->
<Story name="Every protection">
  <div style="display: flex; gap: 1.25rem; align-items: center">
    {#each ["none", "private", "vault", "sealed"] as const as p (p)}
      <span style="display: inline-flex; align-items: center; gap: 0.375rem">
        <Lock protection={p} /><span class="faint" style="font-size: 0.75rem">{p}</span>
      </span>
    {/each}
  </div>
</Story>

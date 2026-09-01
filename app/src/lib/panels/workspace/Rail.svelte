<!--
  The tenancy tree, as the design's left rail.

  Indentation carries the hierarchy, and a space sits at the same depth as a
  project rather than beneath one — because that is what it is: a sibling under
  the organization that *references* projects. Nesting it would draw a parent
  that does not exist.
-->
<script lang="ts">
  import { IconButton, Lock, PanelLabel, TreeNode } from "$lib/ui";
  import { resourceId } from "$lib/api";
  import type { WorkspaceState } from "./state.svelte";

  interface Props {
    /** The tree this rail reads and writes. */
    tree: WorkspaceState;
  }

  let { tree }: Props = $props();
</script>

<nav class="rail">
  <div>
    <PanelLabel>
      Structure
      {#snippet action()}
        <IconButton
          label="New organization"
          onclick={() => {
            // Leaving the space is the point: the create form lives on the
            // tenancy page, and this button did nothing at all while a space
            // was open — it set a field no one read.
            tree.closeSpace();
            tree.startCreating("organization");
          }}
        >
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.2">
            <path d="M6 2v8M2 6h8" />
          </svg>
        </IconButton>
      {/snippet}
    </PanelLabel>

    <div class="tree">
      {#each tree.organizations as organization (organization.name)}
        <TreeNode
          name={organization.displayName}
          kind="org"
          count="{organization.projectCount}p · {organization.spaceCount}s"
          current={tree.selected?.name === organization.name}
          muted={organization.deleted}
          onclick={() => tree.open(organization)}
        />

        {#if tree.selected?.name === organization.name}
          {#each tree.projects as project (project.name)}
            <TreeNode
              name={project.displayName}
              kind="project"
              count={resourceId(project.name)}
              muted={project.deleted}
            />
          {/each}

          {#each tree.spaces as space (space.name)}
            <TreeNode
              name={space.displayName}
              kind="space"
              protection={space.protection}
              current={tree.space?.name === space.name}
              muted={space.deleted}
              onclick={() => tree.openSpace(space)}
            />
          {/each}
        {/if}
      {:else}
        <p class="hint" style="padding: 0 calc(var(--u) * 3.5)">
          No organizations yet. Everything else lives inside one.
        </p>
      {/each}
    </div>
  </div>

  <div>
    <PanelLabel>Legend</PanelLabel>
    <div class="legend">
      <div><Lock protection="none" decorative /> Open · role grants</div>
      <div><Lock protection="vault" decorative /> Vault · key-wrapped</div>
      <div><Lock protection="sealed" decorative /> Sealed · client key</div>
    </div>
  </div>
</nav>

<!--
  The tenancy tree, as the design's left rail.

  Indentation carries the hierarchy — `.node.org`, `.node.project`, `.node.space`
  differ only by left padding — and a space sits at the same depth as a project
  rather than beneath one, because that is what it is: a sibling under the
  organization that *references* projects. Nesting it would draw a parent that
  does not exist.
-->
<script lang="ts">
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
    <div class="rail-label">
      Structure
      <button
        class="rail-add"
        type="button"
        title="New organization"
        onclick={() => tree.startCreating("organization")}
      >
        +
      </button>
    </div>

    <div class="tree">
      {#each tree.organizations as organization (organization.name)}
        <button
          class="node org"
          type="button"
          aria-current={tree.selected?.name === organization.name}
          style={organization.deleted ? "opacity:.5" : ""}
          onclick={() => tree.open(organization)}
        >
          <span class="name">{organization.displayName}</span>
          <span class="count">{organization.projectCount}p · {organization.spaceCount}s</span>
        </button>

        {#if tree.selected?.name === organization.name}
          {#each tree.projects as project (project.name)}
            <div class="node project" style={project.deleted ? "opacity:.5" : ""}>
              <span class="name">{project.displayName}</span>
              <span class="count">{resourceId(project.name)}</span>
            </div>
          {/each}

          {#each tree.spaces as space (space.name)}
            <div class="node space" style={space.deleted ? "opacity:.5" : ""}>
              <span class="name">{space.displayName}</span>
              <!-- The lock is the protection, not a state: an open padlock and a
                   sealed one are different promises, and rule 25 exists because
                   it is tempting to draw them the same. -->
              <span
                class="lock {space.protection === 'sealed'
                  ? 'lock-sealed'
                  : space.protection === 'vault'
                    ? 'lock-shut'
                    : 'lock-open'}"
                title={space.protection}
              >
                {space.protection === "none" ? "○" : space.locked ? "●" : "◐"}
              </span>
            </div>
          {/each}
        {/if}
      {:else}
        <p class="hint" style="padding: 0 0.875rem">
          No organizations yet. Everything else lives inside one.
        </p>
      {/each}
    </div>
  </div>

  <div>
    <div class="rail-label">Legend</div>
    <div class="legend">
      <div><span class="lock lock-open">○</span> Open · role grants</div>
      <div><span class="lock lock-shut">◐</span> Vault · key-wrapped</div>
      <div><span class="lock lock-sealed">●</span> Sealed · client key</div>
    </div>
  </div>
</nav>

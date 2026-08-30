<!--
  Workspace: the tenancy tree, and what one organization holds.

  Three columns, as the design has it — `.view.workspace` is `16rem 1fr 20rem`:
  the rail, the page, and a detail side panel. The rail carries the hierarchy so
  the page never has to repeat it.
-->
<script lang="ts">
  import { client } from "$lib/api";
  import { WorkspaceState } from "./state.svelte";
  import Rail from "./Rail.svelte";
  import Detail from "./Detail.svelte";
  import CreateRow from "./CreateRow.svelte";
  import SpaceRow from "./SpaceRow.svelte";

  // Named `tree` rather than `state`: a local called `state` makes `$state` read
  // as a store subscription on that variable, which Svelte rejects.
  const tree = new WorkspaceState(client);
  void tree.load();
</script>

<div class="view workspace">
  <Rail {tree} />

  <div class="page">
    <div class="page-top">
      <div class="page-inner">
        <div class="page-head">
          <h1>{tree.selected?.displayName ?? "Workspace"}</h1>
          <div class="spacer"></div>
          {#if tree.selected}
            {@const organization = tree.selected}
            <button
              class="btn ghost sm"
              type="button"
              disabled={tree.busy}
              onclick={() => tree.toggleDeleted(organization)}
            >
              {organization.deleted ? "Undelete" : "Delete"}
            </button>
          {/if}
        </div>
      </div>
    </div>

    <div class="page-scroll">
      <div class="page-inner">
        {#if tree.error}
          <p class="selectable" style="color: var(--red-text)">{tree.error}</p>
        {/if}

        {#if tree.creating === "organization" || tree.organizations.length === 0}
          <div>
            <div class="panel-label">New organization</div>
            <CreateRow
              noun="organization"
              busy={tree.busy}
              create={(id, name) => tree.createOrganization(id, name)}
            />
          </div>
        {/if}

        {#if tree.selected}
          <div>
            <div class="panel-label">Projects</div>
            <div class="set-group" style="margin-top: 0.625rem">
              {#each tree.projects as project (project.name)}
                <div class="set-row">
                  <div class="txt"><b>{project.displayName}</b></div>
                  <div class="ctl mono faint" style="font-size: 0.75rem">
                    {project.name}
                  </div>
                </div>
              {:else}
                <div class="set-row"><div class="txt"><span>No projects yet.</span></div></div>
              {/each}
            </div>
            <div style="margin-top: 0.625rem">
              <CreateRow
                noun="project"
                busy={tree.busy}
                create={(id, name) => tree.createProject(id, name)}
              />
            </div>
          </div>

          <div>
            <div class="panel-label">Spaces</div>
            <div class="set-group" style="margin-top: 0.625rem">
              {#each tree.spaces as space (space.name)}
                <SpaceRow {space} />
              {:else}
                <div class="set-row"><div class="txt"><span>No spaces yet.</span></div></div>
              {/each}
            </div>
            <div style="margin-top: 0.625rem">
              <CreateRow noun="space" busy={tree.busy} create={tree.createSpaceWith} />
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>

  <Detail {tree} />
</div>

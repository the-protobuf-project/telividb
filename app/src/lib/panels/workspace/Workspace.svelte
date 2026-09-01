<!--
  The workspace: the tenancy tree, and the space you are working in.

  Three columns, as the design has it — `.view.workspace` is `16rem 1fr 20rem`.
  The rail carries the hierarchy, the centre is the space itself, and the side
  panel holds detail.

  This replaces a version whose centre column was org/project/space CRUD. That
  was the structural mistake: creating a space is something you do occasionally,
  through a form; *working in one* is what the window is for, and it had nowhere
  to happen. Creation now lives in the rail and in the panel that appears when
  no space is open.
-->
<script lang="ts">
  import { Button, Empty, Notice, PanelLabel, SettingGroup, SettingRow } from "$lib/ui";
  import { client } from "$lib/api";
  import { AskState } from "$lib/panels/ask/state.svelte";
  import Space from "$lib/panels/space/Space.svelte";
  import { WorkspaceState } from "./state.svelte";
  import Rail from "./Rail.svelte";
  import Detail from "./Detail.svelte";
  import CreateRow from "./CreateRow.svelte";
  import SpaceRow from "./SpaceRow.svelte";

  interface Props {
    /** Whether the engine can turn text into vectors yet. */
    canEmbed?: boolean;
    /** The resident model's width, stated on each stored point. */
    dimensions?: number;
  }

  let { canEmbed = true, dimensions }: Props = $props();

  // Named `tree` rather than `state`: a local called `state` makes `$state` read
  // as a store subscription on that variable, which Svelte rejects.
  const tree = new WorkspaceState(client);
  void tree.load();

  // One conversation for the panel. It is bound to the space the rail opens; a
  // per-space thread waits on the engine keeping them, which it does not yet.
  const ask = new AskState(client);
  void ask.loadProviders();
</script>

<div class="view workspace">
  <Rail {tree} />

  {#if tree.space}
    <Space
      name={tree.space.displayName}
      protection={tree.space.protection}
      locked={tree.space.locked}
      onclose={() => tree.closeSpace()}
      {ask}
      {canEmbed}
      {dimensions}
    />
  {:else}
    <div class="page">
      <div class="page-top">
        <div class="page-inner">
          <div class="page-head">
            <h1>{tree.selected?.displayName ?? "Workspace"}</h1>
            <div class="spacer" style="flex: 1"></div>
            {#if tree.selected}
              {@const organization = tree.selected}
              <Button
                variant="ghost"
                size="sm"
                loading={tree.busy}
                onclick={() => tree.toggleDeleted(organization)}
              >
                {organization.deleted ? "Undelete" : "Delete"}
              </Button>
            {/if}
          </div>
        </div>
      </div>

      <div class="page-body">
        <div class="page-inner">
          {#if tree.error}<Notice tone="error">{tree.error}</Notice>{/if}

          {#if tree.organizations.length === 0 || tree.creating === "organization"}
            <PanelLabel>New organization</PanelLabel>
            <CreateRow
              noun="organization"
              busy={tree.busy}
              create={async (id, name) => {
                const ok = await tree.createOrganization(id, name);
                // Closed on success only, so a refused name keeps the form and
                // the message together.
                if (ok) tree.creating = null;
                return ok;
              }}
            />
          {:else if tree.selected}
            <Notice>Open a space in the rail to start working in it.</Notice>

            <PanelLabel>Projects</PanelLabel>
            <SettingGroup>
              {#each tree.projects as project (project.name)}
                <SettingRow label={project.displayName} tag={project.deleted ? "deleted" : undefined}>
                  {#snippet control()}
                    <span class="mono faint" style="font-size: 0.75rem">{project.name}</span>
                  {/snippet}
                </SettingRow>
              {:else}
                <SettingRow label="No projects yet" description="Create one below." />
              {/each}
            </SettingGroup>
            <CreateRow
              noun="project"
              busy={tree.busy}
              create={(id, name) => tree.createProject(id, name)}
            />

            <PanelLabel>Spaces</PanelLabel>
            <SettingGroup>
              {#each tree.spaces as space (space.name)}
                <SpaceRow {space} />
              {:else}
                <SettingRow label="No spaces yet" description="Create one below." />
              {/each}
            </SettingGroup>
            <CreateRow noun="space" busy={tree.busy} create={tree.createSpaceWith} />
          {:else}
            <Empty>Nothing selected. Choose an organization in the rail.</Empty>
          {/if}
        </div>
      </div>
    </div>
  {/if}

  <Detail {tree} />
</div>

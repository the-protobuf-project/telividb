<!--
  The tenancy tree: organizations on the left, what one holds on the right.

  Two columns rather than a nested tree because the hierarchy is not one: a space
  is a sibling of a project under the same organization, not a child of it. A
  tree would have to lie about that or invent a parent that does not exist.
-->
<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import * as Select from "$lib/components/ui/select";
  import { resourceId } from "$lib/api";
  import { client } from "$lib/api";
  import type { Protection } from "@telividb/answer";
  import { WorkspaceState } from "./state.svelte";
  import CreateRow from "./CreateRow.svelte";
  import SpaceRow from "./SpaceRow.svelte";

  // Named `tree` rather than `state`: a local called `state` makes `$state`
  // read as a store subscription on that variable, which Svelte rejects.
  const tree = new WorkspaceState(client);
  void tree.load();

  /** Protection for the next space. Fixed once it is created, so chosen first. */
  let protection = $state<Protection>("private");

  const choices: readonly Protection[] = ["none", "private", "vault", "sealed"];
</script>

<div class="flex h-full min-h-0 flex-col">
  <div class="border-border flex shrink-0 items-center justify-between border-b px-6 py-4">
    <h1 class="text-foreground text-lg font-medium">Workspace</h1>
    <CreateRow
      noun="organization"
      busy={tree.busy}
      create={(id, name) => tree.createOrganization(id, name)}
    />
  </div>

  {#if tree.error}
    <p class="text-destructive selectable border-border shrink-0 border-b px-6 py-2 text-sm">
      {tree.error}
    </p>
  {/if}

  <div class="flex min-h-0 flex-1">
    <aside class="border-border w-64 shrink-0 overflow-y-auto border-r p-2">
      {#each tree.organizations as organization (organization.name)}
        <button
          type="button"
          onclick={() => tree.open(organization)}
          class="hover:bg-secondary/60 flex w-full flex-col gap-0.5 px-3 py-2 text-left {tree
            .selected?.name === organization.name
            ? 'bg-secondary'
            : ''} {organization.deleted ? 'opacity-50' : ''}"
        >
          <span class="text-foreground text-sm">{organization.displayName}</span>
          <span class="text-muted-foreground text-xs">
            {organization.projectCount} project{organization.projectCount === 1 ? "" : "s"}
            · {organization.spaceCount} space{organization.spaceCount === 1 ? "" : "s"}
            {organization.deleted ? " · deleted" : ""}
          </span>
        </button>
      {:else}
        <p class="text-muted-foreground px-3 py-6 text-sm">
          No organizations yet. Create one above — everything else lives inside one.
        </p>
      {/each}
    </aside>

    <div class="min-h-0 flex-1 overflow-y-auto p-6">
      {#if tree.selected}
        {@const organization = tree.selected}
        <div class="flex max-w-3xl flex-col gap-8">
          <div class="flex items-center justify-between gap-4">
            <div class="flex flex-col gap-0.5">
              <h2 class="text-foreground text-base font-medium">
                {organization.displayName}
              </h2>
              <span class="text-muted-foreground selectable font-mono text-xs">
                {organization.name}
              </span>
            </div>
            <Button
              size="sm"
              variant={organization.deleted ? "default" : "ghost"}
              disabled={tree.busy}
              onclick={() => tree.toggleDeleted(organization)}
            >
              {organization.deleted ? "Undelete" : "Delete"}
            </Button>
          </div>

          <section class="flex flex-col gap-2">
            <div class="flex items-center justify-between gap-4">
              <h3 class="text-muted-foreground text-xs tracking-wide uppercase">
                Projects
              </h3>
              <CreateRow
                noun="project"
                busy={tree.busy}
                create={(id, name) => tree.createProject(id, name)}
              />
            </div>
            <div class="border-border border">
              {#each tree.projects as project (project.name)}
                <div
                  class="border-border flex items-center justify-between border-b px-3 py-2 last:border-b-0 {project.deleted
                    ? 'opacity-50'
                    : ''}"
                >
                  <span class="text-foreground text-sm">{project.displayName}</span>
                  <span class="text-muted-foreground font-mono text-xs">
                    {resourceId(project.name)}
                  </span>
                </div>
              {:else}
                <p class="text-muted-foreground px-3 py-4 text-sm">No projects yet.</p>
              {/each}
            </div>
          </section>

          <section class="flex flex-col gap-2">
            <div class="flex items-center justify-between gap-4">
              <h3 class="text-muted-foreground text-xs tracking-wide uppercase">
                Spaces
              </h3>
              <div class="flex items-center gap-2">
                <!-- Chosen before the name, because it is the half that cannot
                     be changed afterwards. -->
                <Select.Root type="single" bind:value={protection}>
                  <Select.Trigger class="h-8 w-28 text-xs">{protection}</Select.Trigger>
                  <Select.Content>
                    {#each choices as choice (choice)}
                      <Select.Item value={choice}>{choice}</Select.Item>
                    {/each}
                  </Select.Content>
                </Select.Root>
                <CreateRow
                  noun="space"
                  busy={tree.busy}
                  create={(id, name) => tree.createSpace(id, name, protection)}
                />
              </div>
            </div>
            <div class="border-border border">
              {#each tree.spaces as space (space.name)}
                <SpaceRow {space} />
              {:else}
                <p class="text-muted-foreground px-3 py-4 text-sm">No spaces yet.</p>
              {/each}
            </div>
          </section>
        </div>
      {:else}
        <p class="text-muted-foreground text-sm">
          Nothing selected. Create an organization to begin.
        </p>
      {/if}
    </div>
  </div>
</div>

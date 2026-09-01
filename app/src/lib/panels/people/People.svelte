<!--
  Who can be granted access, and what has been granted.

  # These are records, and nothing consults them

  `telividb-policy` does not exist. A role binding here grants nothing: no query
  reads it, no search is filtered by it, and six invariants describe an
  authorization system with nothing behind it.

  That is stated on the screen rather than only in a comment, because the failure
  mode is specific and severe. A screen listing users and roles reads as a
  working permission system, and someone could reasonably conclude their data is
  protected by it. It is not. This exists so the shape is settled before the
  engine that reads it is written.
-->
<script lang="ts">
  import {
    Button, DataViewport, Empty, Notice, Paged, Pager, PanelLabel, Row, SearchField,
    SettingGroup, SettingRow, Tag, TreeNode,
  } from "$lib/ui";
  import { BINDINGS, GROUPS, PEOPLE } from "./fixture";

  /** Which group's members are showing. */
  let group = $state<string | null>(GROUPS[0]?.name ?? null);

  let chosen = $derived(GROUPS.find((g) => g.name === group));
  let members = $derived(PEOPLE.filter((p) => p.groups.includes(group ?? "")));
  let grants = $derived(BINDINGS.filter((b) => b.group === group));

  /** Searchable by the two things a person is looked up by: name and principal. */
  const list = new Paged(() => members, (m) => [m.displayName, m.principal, m.name], 8);

  /** The last path segment — what a person typed to create it. */
  function id(name: string): string {
    return name.slice(name.lastIndexOf("/") + 1);
  }
</script>

<div class="view two">
  <nav class="rail">
    <PanelLabel>Groups</PanelLabel>
    <div class="tree">
      {#each GROUPS as g (g.name)}
        <TreeNode
          name={g.displayName}
          kind="org"
          count={String(PEOPLE.filter((p) => p.groups.includes(g.name)).length)}
          current={group === g.name}
          onclick={() => (group = g.name)}
        />
      {/each}
    </div>

    <div style="padding: 0 calc(var(--u) * 3.5)">
      <Notice tone="warn">
        Sample data — Identity.ListUsers is not served yet.
      </Notice>
    </div>
  </nav>

  <div class="page">
    <div class="page-top">
      <div class="page-inner">
        <div class="page-head">
          <h1>{chosen?.displayName ?? "People"}</h1>
          <div class="spacer" style="flex: 1"></div>
          <SearchField
            bind:value={list.query}
            noun="people"
            matched={list.matches.length}
            total={members.length}
          />
          <Button
            size="sm"
            disabled
            title="Needs the Identity service, which is not served yet"
          >
            Invite someone
          </Button>
        </div>
      </div>
    </div>

    <!-- Three bands: what is fixed, the members list, then the grants. Members
         is the one unbounded thing here, so it takes what is left and scrolls
         inside itself — at the window's 520px minimum this page was otherwise
         9px too tall and lost the bottom row off the edge. -->
    <div class="page-body">
      <div class="page-inner people-stack">
        <div class="people-band">
          <!-- Before the list, not after it. Someone scanning this screen for
               reassurance should meet the correction first. -->
          <Notice tone="error">
            Nothing here is enforced. These are records: no query reads a role
            binding, and no search is filtered by one. Authorization arrives with
            the policy engine, which is not built.
          </Notice>

          <PanelLabel>Members</PanelLabel>
        </div>

        <DataViewport label="Members">
        {#each list.rows as person (person.name)}
          <Row name={person.displayName}>
            {#snippet badges()}
              {#if !person.principal}
                <!-- A record with no credential behind it is a real state —
                     invited but not arrived — and different from a broken row. -->
                <Tag tone="amber">no principal</Tag>
              {/if}
            {/snippet}
            {#snippet meta()}
              <div class="row-meta mono">
                {person.principal || "no identity has signed in as this user yet"}
              </div>
              <div class="row-meta">
                {person.groups.map(id).join(" · ")}
              </div>
            {/snippet}
          </Row>
        {:else}
          <Empty>
            {list.query.trim() ? `No one matches “${list.query}”.` : "No one in this group."}
          </Empty>
        {/each}
        </DataViewport>

        <div class="people-band">
        {#if list.paged}
          <div style="display: flex; justify-content: flex-end">
            <Pager page={list.page} pages={list.pages} go={(d) => list.go(d)} />
          </div>
        {/if}

        <PanelLabel>Grants</PanelLabel>
        <SettingGroup>
          {#each grants as binding (binding.name)}
            <SettingRow
              label={binding.role}
              description="over {binding.scope}"
              tag="not enforced"
              tone="amber"
            >
              {#snippet control()}
                <span class="mono faint" style="font-size: 0.75rem">{id(binding.name)}</span>
              {/snippet}
            </SettingRow>
          {:else}
            <SettingRow
              label="No grants"
              description="This group has been given nothing — which is the same as every other group, since none of it is read."
            />
          {/each}
        </SettingGroup>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  /* Fixed, flexing, fixed. Only the members list gives ground when the window
     is short, which is what keeps the grants section and the pager on screen
     instead of off the bottom edge. */
  .people-stack {
    display: grid;
    grid-template-rows: auto 1fr auto;
    gap: calc(var(--u) * 4);
    min-height: 0;
    height: 100%;
  }

  .people-band {
    display: flex;
    flex-direction: column;
    gap: calc(var(--u) * 4);
    min-height: 0;
  }
</style>

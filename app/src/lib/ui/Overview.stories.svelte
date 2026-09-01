<!--
  Everything at once.

  Composed from the same components the panels import — not a copy of their
  markup. That is the difference between a design system and a picture of one:
  change `Button.svelte` and this page changes with the app, rather than drifting
  from it silently.

  Each element also has its own story under **Elements**, which is where to go to
  change one.
-->
<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import {
    Bar, Button, Chip, Dot, Field, IconButton, Input, Kv, PanelLabel, Row,
    Seg, SettingGroup, SettingRow, Steps, Tag, Textarea, Tile, Tiles, Toggle,
    TreeNode,
  } from "./index";

  const { Story } = defineMeta({ title: "Design system", parameters: { layout: "fullscreen" } });

  let device = $state("Auto");
  let redact = $state(true);
</script>

<Story name="All">
  <div style="padding: var(--gutter); display: flex; flex-direction: column; gap: 1.25rem">
    <div>
      <span class="wordmark" style="font-size: 1.0625rem">telivi<span>db</span></span>
      <p class="lede" style="margin: 0.375rem 0 0">
        Every element, composed from the components the app imports. Open
        <b>Elements</b> in the sidebar to change one on its own.
      </p>
    </div>

    <PanelLabel>Buttons</PanelLabel>
    <div style="display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap">
      <Button>Primary</Button>
      <Button size="sm">Primary small</Button>
      <Button variant="ghost">Ghost</Button>
      <Button variant="ghost" size="sm">Ghost small</Button>
      <Button disabled>Disabled</Button>
      <IconButton label="New space">
        <svg width="14" height="14" viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.3"><path d="M4 9h10M9 4v10" /></svg>
      </IconButton>
    </div>

    <PanelLabel>Fields</PanelLabel>
    <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(18rem, 1fr)); gap: 0.75rem">
      <Field label="Organization" hint="The id derived from it is permanent.">
        <Input placeholder="Acme Research" />
      </Field>
      <Field label="Question">
        <Textarea rows={2} placeholder="Ask anything…" />
      </Field>
    </div>

    <PanelLabel>Tags, dots and chips</PanelLabel>
    <div style="display: flex; gap: 0.75rem; align-items: center; flex-wrap: wrap">
      <Tag>installed</Tag>
      <Tag tone="green">in memory</Tag>
      <Tag tone="amber">sample data</Tag>
      <Tag tone="blue">recommended</Tag>
      <span style="width: 0.5rem"></span>
      <!-- Named as they would be in use, not by the state word: "live" means
           something different on a model row and on a provider. -->
      <Dot state="idle" title="not configured" />
      <Dot state="live" title="ready" />
      <Dot state="warn" title="needs attention" />
      <Dot state="off" title="failed" />
      <span style="width: 0.5rem"></span>
      <Chip label="bge-small-en-v1.5" state="live" />
      <Chip label="metal" state="live" />
      <Chip label="no model" />
    </div>

    <PanelLabel>Steps</PanelLabel>
    <Steps labels={["Model", "Organization", "Project", "Space"]} current={1} />

    <PanelLabel>Rows</PanelLabel>
    <Row name="BGE Small">
      {#snippet badges()}<Tag tone="blue">recommended</Tag>{/snippet}
      {#snippet meta()}
        <div class="row-meta">Smallest and quickest. 384 dimensions keeps indexes compact.</div>
        <div class="row-meta mono">384 dim · 512 tokens · 37 MB · mit</div>
      {/snippet}
      {#snippet action()}<Button size="sm">Install</Button>{/snippet}
    </Row>
    <Row name="Qwen3 Embedding" selected>
      {#snippet badges()}<Tag tone="green">in memory</Tag>{/snippet}
      {#snippet meta()}
        <div class="row-meta mono">1024 dim · 639 MB</div>
        <div style="margin-top: 0.5rem"><Bar value={0.64} /></div>
      {/snippet}
    </Row>

    <PanelLabel>Tiles</PanelLabel>
    <Tiles>
      <Tile label="Backend" value="metal" note="accelerated" />
      <Tile label="Budget" value="28.8" unit="GB" note="measured" />
      <Tile label="Resident" value="0.6" unit="GB" note="indexes and models" />
      <Tile label="Engine" value="0.1.0" note="build that answered" compact />
    </Tiles>

    <PanelLabel>Setting rows</PanelLabel>
    <SettingGroup>
      <SettingRow label="Data directory" description="Segments, the write-ahead log, models and metadata.">
        {#snippet control()}<span class="mono faint" style="font-size:0.75rem">~/.telividb/data</span>{/snippet}
      </SettingRow>
      <SettingRow label="Compute backend" description="Chosen by detection when the process started.">
        {#snippet control()}<Seg options={["Auto", "Metal", "CPU"]} bind:value={device} label="Compute backend" />{/snippet}
      </SettingRow>
      <SettingRow label="Redact before embedding" description="A vector made from a secret leaks it." tag="not built" tone="amber">
        {#snippet control()}<Toggle bind:pressed={redact} label="Redact before embedding" />{/snippet}
      </SettingRow>
    </SettingGroup>

    <PanelLabel>Tree and detail</PanelLabel>
    <div style="display: grid; grid-template-columns: 16rem 20rem; gap: 1rem">
      <div class="tree" style="border: 1px solid var(--rule)">
        <TreeNode name="Acme Research" kind="org" count="2p · 3s" />
        <TreeNode name="Retrieval" kind="project" count="retrieval" />
        <TreeNode name="Notes" kind="space" protection="none" current />
        <TreeNode name="Journal" kind="space" protection="vault" />
        <TreeNode name="Board" kind="space" protection="sealed" />
      </div>
      <div style="display: flex; flex-direction: column; gap: 0.375rem">
        <Kv label="Points" value="1,284" />
        <Kv label="Edges" value="312" />
        <Kv label="Dimensions" value="384" />
        <Kv label="Metric" value="cosine" />
      </div>
    </div>
  </div>
</Story>

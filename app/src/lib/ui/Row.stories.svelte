<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import Row from "./Row.svelte";
  import Tag from "./Tag.svelte";
  import Bar from "./Bar.svelte";
  import Button from "./Button.svelte";
  import Dot from "./Dot.svelte";

  const { Story } = defineMeta({ title: "Elements/Row" });
  let selected = $state("bge");
</script>

<!-- The list primitive: models, people and collections are all this shape. -->
<Story name="States">
  <div style="max-width: 46rem">
    <Row
      name="BGE Small"
      selected={selected === "bge"}
      onclick={() => (selected = "bge")}
    >
      {#snippet badges()}<Tag tone="blue">recommended</Tag>{/snippet}
      {#snippet meta()}
        <div class="row-meta">Smallest and quickest. 384 dimensions keeps indexes compact.</div>
        <div class="row-meta mono">384 dim · 512 tokens · 37 MB · mit</div>
      {/snippet}
      {#snippet action()}<Button size="sm">Install</Button>{/snippet}
    </Row>

    <Row
      name="Qwen3 Embedding"
      selected={selected === "qwen"}
      onclick={() => (selected = "qwen")}
    >
      {#snippet badges()}<Tag tone="green">in memory</Tag>{/snippet}
      {#snippet meta()}
        <div class="row-meta mono">1024 dim · 32768 tokens · 639 MB</div>
        <div style="margin-top: 0.5rem"><Bar value={0.64} /></div>
      {/snippet}
      {#snippet action()}<Dot state="live" title="On disk" />{/snippet}
    </Row>

    <Row name="Deleted model" muted>
      {#snippet meta()}<div class="row-meta">Soft-deleted, still recoverable.</div>{/snippet}
    </Row>
  </div>
</Story>

<!--
  The catalogue, rebuilt in Tailwind.

  Three states, because the two the mock never drew are the ones a fresh install
  spends its time in: nothing loaded yet, and a search that matches nothing.
-->
<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import ModelRow from "./ModelRow.svelte";
  import type { CatalogModel } from "$lib/api";

  const { Story } = defineMeta({ title: "Models/Row", parameters: { layout: "fullscreen" } });

  const base: CatalogModel = {
    id: "bge-small-en-v1.5",
    displayName: "BGE Small (English)",
    description: "The smallest model here and a good default. 384 dimensions keeps indexes compact.",
    repositoryUri: "https://huggingface.co/CompendiumLabs/bge-small-en-v1.5-gguf",
    sizeBytes: 36_806_944,
    dimensions: 384,
    contextLength: 512,
    license: "mit",
    recommended: true,
    installed: false,
    resident: false,
  } as CatalogModel;
</script>

<Story name="Every state">
  <div class="mx-auto max-w-4xl p-5">
    <ModelRow model={base} onInstall={() => {}} onCancel={() => {}} />
    <ModelRow
      model={{ ...base, id: "b", displayName: "BGE Base", recommended: false, installed: true }}
      onInstall={() => {}}
      onCancel={() => {}}
    />
    <ModelRow
      model={{ ...base, id: "q", displayName: "Qwen3 Embedding", dimensions: 1024, sizeBytes: 639_000_000, recommended: false, installed: true, resident: true }}
      onInstall={() => {}}
      onCancel={() => {}}
    />
    <ModelRow
      model={{ ...base, id: "d", displayName: "Downloading right now", recommended: false }}
      install={{ name: "d", state: "downloading", progressBytes: 41_000_000, totalBytes: 118_000_000, error: "" }}
      onInstall={() => {}}
      onCancel={() => {}}
    />
  </div>
</Story>

<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import Input from "./Input.svelte";
  import Field from "./Field.svelte";
  import Textarea from "./Textarea.svelte";

  const { Story } = defineMeta({ title: "Elements/Input" });
  let name = $state("");
</script>

<!-- Typing here shows the binding working, which is what the slug fields need. -->
<Story name="Bound">
  <div style="max-width: 24rem">
    <Field label="Organization" hint={name ? `organizations/${name.toLowerCase().replace(/[^a-z0-9]+/g, "-")}` : "The id derived from it is permanent."}>
      <Input bind:value={name} placeholder="Acme Research" />
    </Field>
  </div>
</Story>

<Story name="Every form">
  <div style="display: grid; gap: 0.75rem; max-width: 32rem">
    <Input placeholder="Plain" />
    <Input mono placeholder="Monospace — ids, hosts, keys" />
    <Input type="password" placeholder="sk-…" />
    <Input placeholder="Disabled" disabled />
    <Textarea placeholder="Ask anything…" rows={3} />
  </div>
</Story>

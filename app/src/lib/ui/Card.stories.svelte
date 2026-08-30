<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import Card from "./Card.svelte";
  import Stage from "./Stage.svelte";
  import Steps from "./Steps.svelte";
  import Field from "./Field.svelte";
  import Input from "./Input.svelte";
  import Button from "./Button.svelte";

  const { Story } = defineMeta({ title: "Elements/Card", parameters: { layout: "fullscreen" } });
  let name = $state("");
</script>

<!-- The first-run card on its stage: this is step two of onboarding. -->
<Story name="On the stage">
  <Stage>
    <Card>
      {#snippet head()}
        <Steps labels={["Model", "Organization", "Project", "Space"]} current={1} />
      {/snippet}

      <div>
        <h1>Name your organization</h1>
        <p class="lede">
          Everything lives under an organization — projects, spaces and the
          collections behind them.
        </p>
      </div>
      <Field
        label="Organization"
        hint={name ? `organizations/${name.toLowerCase().replace(/[^a-z0-9]+/g, "-")}` : "organizations/…"}
      >
        <Input bind:value={name} placeholder="Acme Research" />
      </Field>

      {#snippet foot()}
        <Button variant="ghost">Back</Button>
        <div style="flex: 1"></div>
        <Button disabled={name.trim() === ""}>Continue</Button>
      {/snippet}
    </Card>
  </Stage>
</Story>

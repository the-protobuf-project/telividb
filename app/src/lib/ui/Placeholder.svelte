<!--
  A panel that is designed but not built.

  It names what it is waiting on. "Coming soon" tells a reader nothing they can
  act on; the service that has to exist first is a fact they can go and check.
-->
<script lang="ts">
  import * as Card from "$lib/components/ui/card";

  interface Props {
    /** The panel's name. */
    title: string;
    /** What has to exist before it can be built. */
    waiting: string;
  }

  let { title, waiting }: Props = $props();
</script>

<div class="flex h-full items-center justify-center p-4">
  <Card.Root class="max-w-md">
    <Card.Header>
      <Card.Title class="text-sm">{title}</Card.Title>
      <Card.Description>{waiting}</Card.Description>
    </Card.Header>
  </Card.Root>
</div>

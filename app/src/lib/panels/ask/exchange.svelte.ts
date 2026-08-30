/**
 * One question, what was retrieved for it, and the answer being written.
 *
 * A class rather than a record because the answer arrives over seconds: the text
 * grows, and the panel re-renders as it does. Retrieval is settled before this
 * exists — the hits are what they are — so only the answering half is `$state`.
 */

import { answer as streamAnswer } from "@telividb/answer";
import type { Passage, Protection, Provider } from "@telividb/answer";
import type { SearchHit, TelividbClient } from "$lib/api";

/** A question and its answer, from retrieval through to the last token. */
export class Exchange {
  /** Answer text so far, growing as it streams. */
  public text = $state("");
  /** Whether tokens are still arriving. */
  public streaming = $state(false);
  /** What went wrong, phrased by whoever refused. */
  public error = $state<string | null>(null);

  public constructor(
    /** What was typed. */
    public readonly question: string,
    /** What the collection returned, best first. */
    public readonly hits: readonly SearchHit[],
  ) {}

  /** The hits that carry text, numbered as the prompt will number them. */
  public get passages(): Passage[] {
    return this.hits
      .filter((h) => h.text !== null)
      .map((h) => ({ id: h.id, text: h.text as string, score: h.score }));
  }

  /**
   * Write the answer, streaming it in.
   *
   * The credential is fetched here and not held: it lives in this function's
   * frame for as long as the request takes and is unreachable afterwards, which
   * is the most this design can offer given that the call is made in the window.
   *
   * A failure lands in {@link error} rather than throwing. Every one of them is a
   * sentence worth showing — no key, a refused provider, a locked vault — and a
   * panel that swallowed them would leave the reader with an empty box.
   */
  public async write(
    client: TelividbClient,
    provider: Provider,
    model: string,
    space: string,
    protection: Protection,
  ): Promise<void> {
    if (this.streaming) return;
    this.streaming = true;
    this.error = null;
    this.text = "";

    try {
      const credential = await client.providerCredential(provider.id);
      const stream = streamAnswer({
        question: this.question,
        space,
        protection,
        passages: this.passages,
        provider,
        model,
        credential,
      });
      for await (const chunk of stream) {
        this.text += chunk.text;
      }
    } catch (cause) {
      this.error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      this.streaming = false;
    }
  }
}

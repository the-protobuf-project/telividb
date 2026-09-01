/**
 * Anthropic.
 *
 * `dangerouslyAllowBrowser` is set because the SDK refuses to construct outside
 * Node without it. The flag exists to stop a key being shipped to the public in
 * a web page; here the page is a local application window and the key came from
 * the OS keychain, so the risk it names is not the risk that applies — the one
 * that does is script injected through a rendered passage, which is noted in
 * `guard.ts` and not solved by declining to set this.
 */

import Anthropic from "@anthropic-ai/sdk";
import { SYSTEM, buildUser } from "../prompt";
import { MAX_TOKENS, TEMPERATURE, type FetchLike } from "../transport";
import type { AnswerChunk, Ask } from "../types";

/** Stream an answer from Anthropic. */
export async function* anthropicAnswer(
  ask: Ask,
  fetchImpl: FetchLike,
): AsyncGenerator<AnswerChunk> {
  const client = new Anthropic({
    apiKey: ask.credential,
    fetch: fetchImpl,
    dangerouslyAllowBrowser: true,
  });

  const stream = client.messages.stream({
    model: ask.model,
    max_tokens: MAX_TOKENS,
    temperature: TEMPERATURE,
    system: SYSTEM,
    messages: [{ role: "user", content: buildUser(ask.question, ask.passages) }],
  });

  for await (const event of stream) {
    if (
      event.type === "content_block_delta" &&
      event.delta.type === "text_delta"
    ) {
      yield { text: event.delta.text };
    }
  }
}

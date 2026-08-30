/**
 * Ollama, on this machine.
 *
 * The only provider a vault will use, so its correctness matters differently
 * from the others: it is the escape hatch that makes "answered locally or not at
 * all" a usable rule rather than a refusal.
 *
 * Reached by address rather than by key, which is why `credential` carries a host
 * here instead of a secret.
 */

import { Ollama } from "ollama/browser";
import { SYSTEM, buildUser } from "../prompt";
import { TEMPERATURE, type FetchLike } from "../transport";
import type { AnswerChunk, Ask } from "../types";

/** Where Ollama listens unless told otherwise. */
const DEFAULT_HOST = "http://localhost:11434";

/** Stream an answer from a model running on this machine. */
export async function* ollamaAnswer(
  ask: Ask,
  fetchImpl: FetchLike,
): AsyncGenerator<AnswerChunk> {
  const client = new Ollama({
    host: ask.credential.trim() || DEFAULT_HOST,
    fetch: fetchImpl,
  });

  const stream = await client.chat({
    model: ask.model,
    stream: true,
    options: { temperature: TEMPERATURE },
    messages: [
      { role: "system", content: SYSTEM },
      { role: "user", content: buildUser(ask.question, ask.passages) },
    ],
  });

  for await (const part of stream) {
    const text = part.message?.content;
    if (text) yield { text };
  }
}

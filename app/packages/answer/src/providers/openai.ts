/**
 * OpenAI, and OpenRouter through the same shape.
 *
 * One adapter for both because OpenRouter serves the OpenAI wire format at a
 * different address — so the difference is a `baseURL`, not a second client.
 */

import OpenAI from "openai";
import { SYSTEM, buildUser } from "../prompt";
import { MAX_TOKENS, TEMPERATURE, type FetchLike } from "../transport";
import type { AnswerChunk, Ask } from "../types";

/** Where OpenRouter serves the OpenAI-shaped API. */
const OPENROUTER_BASE = "https://openrouter.ai/api/v1";

/** Stream an answer from OpenAI or OpenRouter. */
export async function* openaiAnswer(
  ask: Ask,
  fetchImpl: FetchLike,
): AsyncGenerator<AnswerChunk> {
  const client = new OpenAI({
    apiKey: ask.credential,
    fetch: fetchImpl,
    dangerouslyAllowBrowser: true,
    ...(ask.provider.id === "openrouter" ? { baseURL: OPENROUTER_BASE } : {}),
  });

  const stream = await client.chat.completions.create({
    model: ask.model,
    temperature: TEMPERATURE,
    max_tokens: MAX_TOKENS,
    stream: true,
    messages: [
      { role: "system", content: SYSTEM },
      { role: "user", content: buildUser(ask.question, ask.passages) },
    ],
  });

  for await (const part of stream) {
    const text = part.choices[0]?.delta?.content;
    if (text) yield { text };
  }
}

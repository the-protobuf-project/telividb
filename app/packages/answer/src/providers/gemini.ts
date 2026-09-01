/**
 * Gemini.
 *
 * The one adapter that cannot be handed a `fetch`: `GoogleGenAIOptions` exposes
 * `httpOptions` (base URL, headers, timeout) and no client. It works anyway
 * because Google's Generative Language API is built for browser use with an API
 * key and sends CORS headers — so this is the one provider whose reachability
 * depends on the remote service rather than on our transport. If that changes,
 * the fix is a request built by hand here, not a sidecar.
 */

import { GoogleGenAI } from "@google/genai";
import { SYSTEM, buildUser } from "../prompt";
import { MAX_TOKENS, TEMPERATURE } from "../transport";
import type { AnswerChunk, Ask } from "../types";

/** Stream an answer from Gemini. */
export async function* geminiAnswer(ask: Ask): AsyncGenerator<AnswerChunk> {
  const ai = new GoogleGenAI({ apiKey: ask.credential });

  const stream = await ai.models.generateContentStream({
    model: ask.model,
    contents: buildUser(ask.question, ask.passages),
    config: {
      systemInstruction: SYSTEM,
      temperature: TEMPERATURE,
      maxOutputTokens: MAX_TOKENS,
    },
  });

  for await (const chunk of stream) {
    const text = chunk.text;
    if (text) yield { text };
  }
}

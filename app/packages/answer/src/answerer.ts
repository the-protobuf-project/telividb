/**
 * Picking an adapter and running it.
 *
 * The guard runs here rather than in each adapter, so a provider added later
 * inherits the check instead of having to remember it. Note what that is and is
 * not worth: it makes the rule impossible to forget, not impossible to bypass —
 * see `guard.ts`.
 *
 * **Adapters are imported dynamically**, which is not a micro-optimisation: the
 * four SDKs are roughly half a megabyte together, and a static import graph pulls
 * all of them into the bundle — and into memory — to use one. Loading the chosen
 * one on first use also keeps three vendors' code out of a session that never
 * asks a question.
 */

import { mayAnswer } from "./guard";
import { resolveFetch } from "./transport";
import type { AnswerChunk, Ask } from "./types";

/** Stream an answer for one question. */
export async function* answer(ask: Ask): AsyncGenerator<AnswerChunk> {
  mayAnswer(ask.space, ask.protection, ask.provider);

  const fetchImpl = await resolveFetch();

  switch (ask.provider.id) {
    // OpenRouter serves the OpenAI wire format, so the difference is a base URL
    // rather than a second adapter.
    case "openai":
    case "openrouter": {
      const { openaiAnswer } = await import("./providers/openai");
      yield* openaiAnswer(ask, fetchImpl);
      return;
    }
    case "anthropic": {
      const { anthropicAnswer } = await import("./providers/anthropic");
      yield* anthropicAnswer(ask, fetchImpl);
      return;
    }
    case "gemini": {
      const { geminiAnswer } = await import("./providers/gemini");
      yield* geminiAnswer(ask);
      return;
    }
    case "ollama": {
      const { ollamaAnswer } = await import("./providers/ollama");
      yield* ollamaAnswer(ask, fetchImpl);
      return;
    }
    default:
      // Never a fallback to another provider: a silent substitution would send
      // the passages somewhere the person did not choose.
      throw new Error(
        `No adapter for provider ${ask.provider.id}. It is in the table but its ` +
          `request shape has not been written.`,
      );
  }
}

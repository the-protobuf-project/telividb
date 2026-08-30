# @telividb/answer

The last step of retrieval-augmented answering: a question, the passages found
for it, and the model that writes the prose.

It is a package rather than a folder in the app for two reasons. It has no
Svelte and no Tauri in it — `@tauri-apps/plugin-http` is an optional peer, used
only to sidestep CORS when a window is available — so the browser build that
serves the Linux daemon consumes exactly the same code as the desktop app. And
`prompt` and `guard` are pure functions that decide what a model is told and
what it may be told, which is the part most worth testing and the part hardest
to reach through a SvelteKit harness.

## What is here

| file | what it decides |
|---|---|
| `prompt.ts` | what the model is instructed to do, and how passages are numbered |
| `guard.ts` | whether protected content may be sent to a given provider |
| `transport.ts` | which `fetch` the SDKs use, and why that differs by target |
| `providers/` | one adapter per official SDK, each streaming |
| `answerer.ts` | picking an adapter, after running the guard |

## The guard is advisory

`mayAnswer` runs in the window, and a check in the window can be bypassed by a
modified frontend or by script injected through a rendered passage. It is here so
the rule is explicit and the honest mistake is caught — it is not what would make
a vault a vault. The enforcement that would is server-side: an engine that
declines to *return* protected passages for a search bound to a remote model.
`telividb_providers::may_answer` is that half, written and unwired.

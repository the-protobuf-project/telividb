# Commercial licence

telividb is dual-licensed.

| You are | Your licence | Cost |
|---|---|---|
| Building open-source software whose licence is compatible with AGPL-3.0 | [AGPL-3.0-or-later](./LICENSE) | Free, forever |
| Building proprietary or closed-source software | Commercial licence | Paid |
| Embedding telividb in a product you distribute without source | Commercial licence | Paid |
| Offering telividb, or a service built on it, over a network without publishing your source | Commercial licence | Paid |

The client SDKs and the `.proto` definitions are **Apache-2.0**, so writing a
proprietary application that *talks to* an telividb server needs no commercial
licence. The distinction is linking against or modifying the engine itself.

---

## Why it works this way

The AGPL does the sorting, so no one has to argue about what counts as "open
source" or "proprietary". If your project's licence is AGPL-compatible you
already comply and owe nothing. If your product is closed, the AGPL is
incompatible with how you ship, and a commercial licence is the way through.

The intent is straightforward: **free for people building in the open, paid for
people building on top of it commercially without giving anything back.**

## What the AGPL asks of you

Section 13 is the part most people have not read. If you modify telividb and let
users interact with it over a network, you must offer those users the source of
your modified version — even if you never distribute a binary. That closes the
gap the plain GPL leaves open for hosted services, and it is deliberate.

Note that **static linking counts.** telividb is designed to be embedded, and a
Rust binary that links the engine is a derived work. If that binary is
proprietary, you need a commercial licence.

## What a commercial licence grants

- The right to embed, modify and distribute telividb in closed-source products
- No obligation to publish your source under Section 13
- Terms negotiated per deployment: seats, nodes, redistribution, support

## Getting one

Open an issue titled `commercial licence` or contact the maintainer through the
repository. Include roughly what you are building and how you intend to deploy —
that is usually enough to size it.

---

**This file describes the offer. It is not itself a contract**, and it does not
grant rights. An executed commercial agreement does. Nothing here is legal
advice; if the licensing boundary matters to your business, have a lawyer read
both this and the AGPL before you rely on either.

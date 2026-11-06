# Rules

Every rule in force, one line each, for building and reviewing. Read the section you're working in; open the decision record a rule cites only when you need its reasons. `0004 #5` means rule 5 of [ADR 0004](decisions/0004-architecture.md). The decision records are the source of truth. A change that accepts or supersedes a decision updates this file in the same pull request.

## Crates

- Depend only as ADR 0004's crate table allows; add a dependency in the change that first uses it. (0004 #1, #2)
- Never make one adapter crate depend on another. (0004 #3)
- Use `tokio` in `application` only as a dev-dependency, to run tests. (0009)
- Keep `serde` out of `domain`: domain types never cross the wire. (0004 #4)
- Put a change where ADR 0004's "Where a change goes" table says; if no row fits, write a decision record first. (0004)

## Domain

- Make every game decision in a `domain` function. (0004 #8)
- Never read the clock or generate randomness or ids in `shared`, `domain` or `application`; ask a port. (0004 #12)
- Never let an id or player-readable text be blank; check once, where the type is built. (0007)
- Name a constructor's error `<Type>Error`, an operation's input `<Verb><Noun>`, its success after the operation, and its error `<Operation>Error`. (0008)
- Give each fallible constructor or operation its own error type, listing only what it can return. (0008)
- Never judge the work: accept the agent's pass or fail per criterion, with no evidence. (0006 #4)
- Refuse to approve a step without a handoff note. (0006 #11)
- Resolve a setting as player over project over game. (0006 #8)

## Use cases

- Write one use case per file: `application/src/use_cases/<verb>_<noun>.rs`. (0004 #5)
- Define exactly `Input`, `Output`, `Error` and one public `run` returning `Result<Output, Error>`. (0004 #6)
- Never call another use case; put shared logic in `domain`. (0004 #7)
- Load through ports, call the domain, save through ports; never compare or count port or domain values. (0004 #8)
- Use no numeric or duration literals other than `0` and `1` in `application`. (0004 #9)
- Take a `PlayerId` in `Input`, never a token or session, unless the use case is authentication. (0004 #24)
- Keep `application/src/` to `lib.rs`, `ports/`, `use_cases/` and `testkit/`. (0004 #21)

## Errors

- Give `Error` variants data (ids, counts), never sentences. (0004 #13)
- Write caller-facing wording only in an inbound adapter, in one exhaustive `match` over a use case's `Error`. (0004 #13)

## Ports and tests

- Define each port as a trait in `application/src/ports/<concern>.rs`, one concern per file, using only `domain` and `application` types. (0004 #10)
- Write ports as async traits with `async-trait`; use cases take them as `&dyn Port`. (0009)
- Give each port one suite and one fake in `application/src/testkit/<concern>.rs`; run the suite against the fake and every `outbound-*` implementation. (0004 #11)
- Refuse a save when the state changed since it was loaded, and test that refusal in the port's suite. (0004 #25)
- Write use-case tests in the use case's file, on testkit fakes, never on an adapter. (0004 "Tests")
- Turn on the `testkit` feature only in tests: `application`'s own, and `outbound-*` `[dev-dependencies]`. (0004 #23)

## Adapters

- Call only use cases from inbound adapters, never ports. (0004 #14)
- Expose use cases in one file per inbound crate: `inbound-<transport>/src/bindings.rs`. (0004 #15)
- Authenticate first in each inbound adapter (for MCP, `authenticate_agent_connection`) and pass the `PlayerId` on. (0004 #24)
- Only implement ports in outbound adapters; branch only on the technology (encoding, errors, retries, pagination). (0004 #16)
- Keep every numeric or duration literal in an adapter in a named `const`. (0004 #9)
- Only build adapters and start a runtime in `app`, one file per runtime in `app/src/bin/`. (0004 #17)
- Serve one router from two thin entry points, a Lambda binary and a plain server; no API Gateway. (0003)

## Shared code

- Give code used in two or more modules the one home ADR 0004's "Shared code" table gives it. (0004 #18)
- Keep game vocabulary out of `shared`. (0004 #19, #22)
- Never name a module or file `util`, `utils`, `helper`, `helpers`, `common`, `misc` or `shared` outside `shared`. (0004 #20)

## Commits

- Write Conventional Commits; the scope is the crate's name when a change stays in one crate. (0005)
- Before every commit, run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. (AGENTS.md)
- Close a decision ticket only once its record is committed. (0001)

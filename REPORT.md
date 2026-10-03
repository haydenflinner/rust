# Sound parser snapshot (continues rust#162593)

**Problem:** Kobzol's "clone only the immediate parent" change is unsound. With nested proc-macro invisible delimiters, `m!(None[None[1]] + 2)` captured as `$e:expr` replays as `1 <eof> <eof>` instead of `1 + 2`.

**Fix:** `TokenCursor::clone_for_capture` keeps enclosing cursors up to and including the first visible delimiter. In ordinary code that is just the parent, with no allocation.

**Checks:**
- The reproducer below passes.
- UI suite: 21,995 passed, 0 failed.
- Differential audit replaying every capture against a full-stack snapshot: 3.37M checks over the UI suite and the rustc-perf crates, 0 divergences. This is evidence over that corpus, not a proof.

**Perf:** local `instructions:u` vs base a8a1e6f, Kobzol's version in parentheses.

| | This branch | Kobzol |
|---|---|---|
| Primary geomean | -0.074% | -0.081% |
| Secondary geomean | -0.110% | -0.125% |
| deep-vector | -2.7% | -3.0% |

<details><summary>Reproducer</summary>

Build with `rustc --edition 2021 --crate-type proc-macro nest.rs`, then `rustc --edition 2021 main.rs --extern nest=libnest.so`.

```rust
// nest.rs
// Proc macro that hands a macro_rules! matcher an expression whose first
// token sits two levels deep inside proc-macro-origin (skipped) invisible groups:
//   m!( None[ None[ 1 ] ] + 2 )
extern crate proc_macro;
use proc_macro::*;

fn none(ts: TokenStream) -> TokenTree { Group::new(Delimiter::None, ts).into() }

#[proc_macro]
pub fn emit(input: TokenStream) -> TokenStream {
    let mac: TokenStream = input; // name of the macro_rules! macro to call
    let one: TokenStream = TokenTree::from(Literal::i32_unsuffixed(1)).into();
    let inner = none(TokenStream::from(none(one)));
    let mut args = TokenStream::from(inner);
    args.extend("+ 2".parse::<TokenStream>().unwrap());
    let mut out = mac;
    out.extend("!".parse::<TokenStream>().unwrap());
    out.extend(TokenStream::from(TokenTree::from(Group::new(Delimiter::Parenthesis, args))));
    out
}

// Echoes its input as a string literal, so we can see exactly which tokens a
// captured nonterminal carried.
#[proc_macro]
pub fn show(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    TokenTree::from(Literal::string(&s)).into()
}
```

```rust
// main.rs
extern crate nest;
// `$e:expr` is force-collected, so its tokens are replayed from the
// cursor snapshot when re-emitted.
macro_rules! via_stringify { ($e:expr) => { stringify!($e) } }
macro_rules! via_pm { ($e:expr) => { nest::show!($e) } }
macro_rules! value { ($e:expr) => { $e } }
fn main() {
    let a: &str = nest::emit!(via_stringify);
    let b: &str = nest::emit!(via_pm);
    let c: i32 = nest::emit!(value);
    println!("stringify: {a:?}\nproc-macro: {b:?}\nvalue: {c}");
    assert_eq!(c, 3);
}
```
</details>

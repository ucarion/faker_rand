# faker_rand ![Crates.io](https://img.shields.io/crates/v/faker_rand) ![Docs.rs](https://docs.rs/faker_rand/badge.svg)

`faker_rand` is a Rust crate that lets you easily generate fake data using the
[`rand`](https://docs.rs/rand) crate. It also provides macros so you can easily
build your own data generators on top of those provided out of the box.

## Installation

You can use `faker_rand` in your Rust project by adding the following to your
`Cargo.toml`:

```toml
faker_rand = "0.1"
```

## Usage

See [the docs on docs.rs for more details](https://docs.rs/faker_rand), but at a
high level here's how you can use this crate:

```rust
use rand::Rng;
use faker_rand::en_us::names::FirstName;

// you can display generators using "{}"
println!("random first name: {}", rand::random::<FirstName>());
println!("random first name: {}", rand::thread_rng().gen::<FirstName>());

// or, you can use to_string as well
let name = rand::random::<FirstName>().to_string();
println!("random first name: {}", name);
```

You can also build your own generators on top of those provided by this crate,
for example if you already have a file of fake data you'd like to generate:

```rust
use faker_rand::faker_impl_from_file;

// First, declare your newtype wrapper around String.
struct Demo(String);

// Then, use the macro. data/lorem_words is a path to a file containing
// example words; you will need to change this path to suit your needs.
faker_impl_from_file!(Demo, "data/lorem_words");

use rand::{Rng, SeedableRng};
let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);

assert_eq!("impedit", rng.gen::<Demo>().to_string());
```

Or, if you want to compose upon sub-generators:

```rust
use faker_rand::faker_impl_from_templates;

// First, declare your newtype wrapper around String.
struct Demo(String);

// Then, invoke the macro.
//
// Note well: all commas and semicolons in this example, even trailing
// semicolons, are strictly required.
faker_impl_from_templates! {
    // The type we're creating a generator implementation for.
    Demo;

    // The template patterns.
    "{}.{}", faker_rand::util::AsciiDigit, faker_rand::lorem::Word;
    "{} ~~~ {}", faker_rand::lorem::Word, faker_rand::util::AsciiDigit;
}

use rand::{Rng, SeedableRng};
let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0);

assert_eq!("qui ~~~ 5", rng.gen::<Demo>().to_string());
assert_eq!("debitis ~~~ 5", rng.gen::<Demo>().to_string());
assert_eq!("5.aliquid", rng.gen::<Demo>().to_string());
assert_eq!("0.doloribus", rng.gen::<Demo>().to_string());
```

Most of the documentation for this crate lives in the Rust docs, not this
README. Check out https://docs.rs/faker_rand for copy-pastable examples you can
use.

## Prior Art

If you're considering using this crate, please be aware of prior art in this
space. The most widely-used alternative to this crate is
[`fake`](https://crates.io/crates/fake).

`faker_rand` was created because the author needed:

1. Types that specifically implemented `rand`'s `Distribution` trait (to take
   advantage of blanket `Distribution` implementations). `fake` instead uses a
   custom `Faker` trait, and drives its own calls to `rand`, whereas
   `faker_rand` has `rand` call into it.

2. Macros for creating custom data fakers, making it easier to create one-off
   data generators without having to write much code.

3. Something with few moving parts (`faker_rand` exposes no new traits or
   functions, only types that implement existing traits), so that generators are
   consistent to use and document.

The biggest shortcoming to this crate as opposed to `fake` is that `fake` has
more data generators than `faker_rand`. If there is a data generator you'd like
in `faker_rand` that is not currently available, please open an issue on GitHub!

# Calculation of Wigner symbols and related constants

This crate computes Wigner 3j coefficients and Clebsch-Gordan coefficients in
pure Rust. The calculation is based on the prime factorization of the different
factorials involved in the coefficients, keeping the values in a rational root
form (`sign * \sqrt{s / n}`) for as long as possible. This idea for the
algorithm is described in:

[H. T. Johansson and C. Forssén, SIAM Journal on Scientific Compututing 38 (2016) 376-384](https://doi.org/10.1137/15M1021908)

This implementation takes a lot of inspiration from the
[WignerSymbols](https://github.com/Jutho/WignerSymbols.jl/) Julia implementation
(and even started as a direct translation of it), many thanks to them! This
crate is available under the same license as the Julia package.

## Usage

Add this crate to your `Cargo.toml` dependencies section:

```toml
wigners = "0.1.0"
```

And then call one of the exported function:

```rust
let w3j = wigners::wigner_3j(j1, j2, j3, m1, m2, m3);

let cg = wigners::clebsch_gordan(j1, m1, j2, m1, j3, m3);
```

## Limitations

Only Wigner 3j symbols for full integers (no half-integers) are implemented,
since that's the only part I need for my own work.

6j and 9j symbols can also be computed with this approach; and support for
half-integers should be feasible as well. I'm open to pull-request implementing
these!

## Benchmarks

This benchmark measure the time to compute all possible Wigner 3j symbols up to
a fixed maximal angular momentum.

| angular momentum | wigners (this crate) | wigner-symbols v0.5 | WignerSymbols.jl v2.0 | wigxjpf v1.11 |
|------------------|----------------------|---------------------|-----------------------|---------------|
| 4                | 0.925 ms             | 17.5 ms             | 2.31 ms               | 0.348 ms      |
| 8                | 5.18 ms              | 151 ms              | 12.0 ms               | 2.40 ms       |
| 12               | 14.0 ms              | 595 ms              | 23.0 ms               | 8.21 ms       |
| 20               | 55.0 ms              | 3772 ms             | 88.3 ms               | 43.0 ms       |

## Comparison to `wigner-symbols`

There is another Rust implementation of wigner symbols: the
[wigner-symbols](https://github.com/Rufflewind/wigner-symbols-rs) crate.
`wigner-symbols` also implements 6j and 9j symbols, but it was not usable for my
case since it relies on [rug](https://crates.io/crates/rug) for arbitrary
precision integers and through it on the [GMP](https://gmplib.org/) library. The
GMP library might be problematic for you for one of these reason:
- it is relatively slow (see the benchmarks above)
- it is distributed under LGPL (this crate is distributed under Apache/MIT);
- it is written in C and C++; and as such is hard to cross-compile or compile to WASM;
- it does not support the MSVC compiler on windows, only the GNU compilers

However, while this crate should be able to compute winger 3j coefficients up to
relatively high angular momentum, it does not use arbitrary precision integers
and might fail for very high value. This crate was validated up to l=100, which
is more than enough for my use case.

## License

This crate is distributed under both the MIT license and the Apache 2.0 license.

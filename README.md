# Calculation of Wigner symbols and related constants

This package computes Wigner 3j coefficients and Clebsch-Gordan coefficients in
pure Rust. The calculation is based on the prime factorization of the different
factorials involved in the coefficients, keeping the values in a rational root
form (`sign * \sqrt{s / n}`) for as long as possible. This idea for the
algorithm is described in:

[H. T. Johansson and C. Forssén, SIAM Journal on Scientific Compututing 38 (2016) 376-384](https://doi.org/10.1137/15M1021908)

This implementation takes a lot of inspiration from the
[WignerSymbols](https://github.com/Jutho/WignerSymbols.jl/) Julia implementation
(and even started as a direct translation of it), many thanks to them! This
package is available under the same license as the Julia package.

## Usage

### From python

```
pip install wigners
```

And then call one of the function:

```py
from  wigners import wigner_3j, clebsch_gordan

w3j = wigner_3j(j1, j2, j3, m1, m2, m3)

cg = clebsch_gordan(j1, m1, j2, m1, j3, m3)
```

### From rust

Add this crate to your `Cargo.toml` dependencies section:

```toml
wigners = "0.1"
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
a fixed maximal angular momentum; clearing up any cached values from previous
angular momentum before starting the loop. In pseudo code, the benchmark looks
like this:

```
if cached_wigner_3j:
    clear_wigner_3j_cache()

# only measure the time taken by the loop
start = time.now()
for j1 in range(max_angular):
    for j2 in range(max_angular):
        for j3 in range(max_angular):
            for m1 in range(-j1, j1 + 1):
                for m2 in range(-j2, j2 + 1):
                    for m3 in range(-j3, j3 + 1):
                        w3j = wigner_3j(j1, j2, j3, m1, m2, m3)

elapsed = start - time.now()
```

| angular momentum | wigners (this) | wigner-symbols v0.5 | WignerSymbols.jl v2.0 | wigxjpf v1.11 | sympy v1.9 |
|------------------|----------------|---------------------|-----------------------|---------------|------------|
| 4                | 0.470 ms       | 28.2 ms             | 3.08 ms               | 0.478 ms      | 83.8 ms    |
| 8                | 9.84 ms        | 867 ms              | 66.2 ms               | 14.5 ms       | 3.50 s     |
| 12               | 73.4 ms        | 7.35 s              | 698 ms                | 122 ms        | 64.2 s     |
| 16               | 342 s          | 36.2 s              | 3.20 s                | 624 ms        |    /       |
| 20               | 1.14 s         |   /                 | 11.1 s                | 2.18 s        |    /       |

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

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

And then call one of the exported function:

```py
import wigners

w3j = wigners.wigner_3j(j1, j2, j3, m1, m2, m3)

cg = wigners.clebsch_gordan(j1, m1, j2, m1, j3, m3)

# full array of Clebsch-Gordan coefficients, computed in parallel
cg_array = wigners.clebsch_gordan_array(ji, j2, j3)

# we have an internal cache for recently computed CG coefficients, if you
# need to clean it up you can use this function
wigners.clear_wigner_3j_cache()
```

### From rust

Add this crate to your `Cargo.toml` dependencies section:

```toml
wigners = "0.3"
```

And then call one of the exported function:

```rust
let w3j = wigners::wigner_3j(j1, j2, j3, m1, m2, m3);

let cg = wigners::clebsch_gordan(j1, m1, j2, m1, j3, m3);

wigners::clear_wigner_3j_cache();
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

Here are the results on an Apple M1 Max (10 cores) CPU, against a handful of
other libraries:

- wigner-symbols: https://crates.io/crates/wigner-symbols
- WignerSymbols.jl: https://github.com/Jutho/WignerSymbols.jl
- wigxjpf: http://fy.chalmers.se/subatom/wigxjpf/
- 0382/WignerSymbol: https://github.com/0382/WignerSymbol
- sympy: https://docs.sympy.org/latest/modules/physics/quantum/cg.html

| code & version             | max_angular=4 |    8    |   12    |   16    |   20   |
|----------------------------|---------------|---------|---------|---------|--------|
| wigners (this)             |  0.190 ms     | 4.60 ms | 36.5 ms | 172 ms  | 572 ms |
| wigner-symbols v0.5        |  6.00 ms      | 181 ms  | 1.53 s  | 7.32 s  |   /    |
| WignerSymbols.jl v2.0      |  1.09 ms      | 21.1 ms | 179 ms  | 902 ms  | 3.09 s |
| wigxjpf v1.11              |  0.237 ms     | 7.65 ms | 68.3 ms | 342 ms  | 1.24 s |
| 0382/WignerSymbol vf8c8dce |  0.070 ms     | 2.26 ms | 19.3 ms | 93.5 ms | 320 ms |
| sympy v1.11                |  24.8 ms      | 1.15 s  | 20.8 s  |    /    |   /    |


A second set of benchmarks checks computing Wigner symbols for large `j`, with the
corresponding `m` varying from -10 to 10, i.e. in pseudo code:

```
if cached_wigner_3j:
    clear_wigner_3j_cache()

# only measure the time taken by the loop
start = time.now()
for m1 in range(-10, 10 + 1):
    for m2 in range(-10, 10 + 1):
        for m3 in range(-10, 10 + 1):
            w3j = wigner_3j(j1, j2, j3, m1, m2, m3)

elapsed = start - time.now()
```

| code & version             | j1=300, j2=100, j3=250  |
|----------------------------|-------------------------|
| wigners (this)             |  29.2 ms                |
| wigner-symbols v0.5        |  13.8 ms                |
| WignerSymbols.jl v2.0      |  11.5 ms                |
| wigxjpf v1.11              |  7.45 ms                |
| 0382/WignerSymbol vf8c8dce |  / (wrong result)       |
| sympy v1.11                |  2.34 s                 |


To run the benchmarks yourself on your own machine, execute the following commands:

```bash
cd benchmarks
cargo bench # this gives the results for wigners, wigner-symbols, wigxjpf and 0382/WignerSymbol

python sympy-bench.py # this gives the results for sympy

julia wigner-symbol.jl # this gives the results for WignerSymbols.jl
```

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

As you can see in the benchmarks above, this usage of GMP becomes an advantage
for large j, where the algorithm used in this crate does not scale as well.

## License

This crate is distributed under both the MIT license and the Apache 2.0 license.

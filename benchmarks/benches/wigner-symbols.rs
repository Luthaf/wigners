#![allow(clippy::needless_return)]

use wigner_symbols::Wigner3jm;

use criterion::{Criterion, criterion_group, criterion_main};

fn bench_wigner3j(c: &mut Criterion) {
    let mut group = c.benchmark_group("wigner-symbols");
    group.sample_size(10);
    group.sampling_mode(criterion::SamplingMode::Flat);

    for &max_angular in &[4, 8, 12, 16] {
        group.bench_function(&format!("max_angular={}", max_angular), |b| {
            b.iter(|| {
                for l1 in 0..=max_angular {
                    for l2 in 0..=max_angular {
                        for l3 in 0..=max_angular {
                            for m1 in -l1..=l1 {
                                for m2 in -l2..=l2 {
                                    for m3 in -l3..=l3 {
                                        let symbol = Wigner3jm{
                                            tj1: 2 * l1,
                                            tj2: 2 * l2,
                                            tj3: 2 * l3,
                                            tm1: 2 * m1,
                                            tm2: 2 * m2,
                                            tm3: 2 * m3,
                                        };
                                        let _ = f64::from(symbol.value());
                                    }
                                }
                            }
                        }
                    }
                }
            })
        });
    }
}

criterion_group!(wigner3j, bench_wigner3j);
criterion_main!(wigner3j);

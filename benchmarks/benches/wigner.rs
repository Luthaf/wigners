#![allow(clippy::needless_return)]

use std::time::Duration;

use wigners::wigner_3j;

use criterion::{Criterion, criterion_group, criterion_main};

fn bench_wigner3j(c: &mut Criterion) {
    let mut group = c.benchmark_group("wigners");
    group.sample_size(10);
    group.sampling_mode(criterion::SamplingMode::Flat);
    group.warm_up_time(Duration::from_secs(1));

    for &max_angular in &[4, 8, 12, 16, 20] {
        group.bench_function(&format!("max_angular={}", max_angular), |b| {
            b.iter(|| {
                for l1 in 0..=max_angular {
                    for l2 in 0..=max_angular {
                        for l3 in 0..=max_angular {
                            for m1 in -l1..=l1 {
                                for m2 in -l2..=l2 {
                                    for m3 in -l3..=l3 {
                                        wigner_3j(l1 as u32, l2 as u32, l3 as u32, m1, m2, m3);
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

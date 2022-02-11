#![allow(clippy::needless_return)]

use std::time::{Duration, Instant};

use wigners::wigner_3j;

use criterion::{Criterion, criterion_group, criterion_main};

fn compute_all_wigner_3j(max_angular: i32) {
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
}

fn bench_wigner3j(c: &mut Criterion) {
    let mut group = c.benchmark_group("wigners");
    group.sample_size(10);
    group.sampling_mode(criterion::SamplingMode::Flat);
    group.warm_up_time(Duration::from_secs(1));

    for &max_angular in &[4, 8, 12,16, 20] {
        group.bench_function(&format!("max_angular={}", max_angular), |b| {
            b.iter_custom(|n_iters| {
                let mut duration = Duration::new(0, 0);
                for _ in 0..n_iters {
                    wigners::clear_wigner_3j_cache();

                    // only benchmark `compute_all_wigner_3j`, not including
                    // previously filled cache
                    let start = Instant::now();
                    compute_all_wigner_3j(max_angular);
                    duration += start.elapsed();
                }

                return duration
            })
        });
    }
}

criterion_group!(wigner3j, bench_wigner3j);
criterion_main!(wigner3j);

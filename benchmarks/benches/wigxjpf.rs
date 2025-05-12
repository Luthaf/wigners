#![allow(clippy::needless_return)]
use std::time::{Duration, Instant};

use wigner_benchmarks::wigxjpf;

use criterion::{Criterion, criterion_group, criterion_main};

fn compute_all_wigner_3j(max_angular: i32) {
    for l1 in 0..=max_angular {
        for l2 in 0..=max_angular {
            for l3 in 0..=max_angular {
                for m1 in -l1..=l1 {
                    for m2 in -l2..=l2 {
                        for m3 in -l3..=l3 {
                            unsafe {
                                wigxjpf::wig3jj(
                                    2 * l1, 2 * l2, 2 * l3,
                                    2 * m1, 2 * m2, 2 * m3
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}


fn compute_large_wigner_3j(l1: i32, l2: i32, l3: i32) {
    for m1 in -10..=10 {
        for m2 in -10..=10 {
            for m3 in -10..=10 {
                unsafe {
                    wigxjpf::wig3jj(
                        2 * l1, 2 * l2, 2 * l3,
                        2 * m1, 2 * m2, 2 * m3
                    );
                }
            }
        }
    }
}


fn bench_wigner3j(c: &mut Criterion) {
    let mut group = c.benchmark_group("wigxjpf");
    group.sample_size(10);
    group.sampling_mode(criterion::SamplingMode::Flat);

    for &max_angular in &[4, 8, 12, 16, 20] {
        group.bench_function(format!("max_angular={}", max_angular), |b| {
            b.iter_custom(|n_iters| {
                let mut duration = Duration::new(0, 0);
                for _ in 0..n_iters {
                    unsafe {
                        wigxjpf::wig_table_init(2 * 100, 6);
                        wigxjpf::wig_temp_init(2 * 100);
                    }

                    // only benchmark `compute_all_wigner_3j`, not wig_table
                    // setup & teardown
                    let start = Instant::now();
                    compute_all_wigner_3j(max_angular);
                    duration += start.elapsed();

                    unsafe {
                        wigxjpf::wig_temp_free();
                        wigxjpf::wig_table_free();
                    }
                }

                return duration
            })
        });
    }

    group.bench_function("j = (300, 100, 250)", |b| {
        b.iter_custom(|n_iters| {
            let mut duration = Duration::new(0, 0);
            for _ in 0..n_iters {
                unsafe {
                        wigxjpf::wig_table_init(2 * 300, 6);
                        wigxjpf::wig_temp_init(2 * 300);
                    }

                    // only benchmark `compute_all_wigner_3j`, not wig_table
                    // setup & teardown
                    let start = Instant::now();
                    compute_large_wigner_3j(300, 100, 250);
                    duration += start.elapsed();

                    unsafe {
                        wigxjpf::wig_temp_free();
                        wigxjpf::wig_table_free();
                    }
            }

            return duration
        })
    });
}

criterion_group!(wigner3j, bench_wigner3j);
criterion_main!(wigner3j);

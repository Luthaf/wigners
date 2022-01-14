#![allow(clippy::needless_return)]

use wigner_benchmarks::wigxjpf;

use criterion::{Criterion, criterion_group, criterion_main};

fn bench_wigner3j(c: &mut Criterion) {
    unsafe {
        wigxjpf::wig_table_init(100, 9);
        wigxjpf::wig_temp_init(100);
    }

    for &max_angular in &[4, 8, 12, 20] {
        c.bench_function(&format!("wigxjpf={}", max_angular), |b| {
            b.iter(|| {
                for l1 in 0..=max_angular {
                    for l2 in 0..=max_angular {
                        for l3 in 0..=3 {
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
            })
        });
    }

    unsafe {
        wigxjpf::wig_temp_free();
        wigxjpf::wig_table_free();
    }
}

criterion_group!(wigner3j, bench_wigner3j);
criterion_main!(wigner3j);

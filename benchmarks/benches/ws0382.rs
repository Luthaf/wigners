#![allow(clippy::needless_return)]

use wigner_benchmarks::wigner_symbol_0382;

use criterion::{Criterion, criterion_group, criterion_main};

fn compute_all_wigner_3j(max_angular: i32) {
    for l1 in 0..=max_angular {
        for l2 in 0..=max_angular {
            for l3 in 0..=max_angular {
                for m1 in -l1..=l1 {
                    for m2 in -l2..=l2 {
                        for m3 in -l3..=l3 {
                            unsafe {
                                wigner_symbol_0382::ws0382_wigner_3j(
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
                    wigner_symbol_0382::ws0382_wigner_3j(
                        2 * l1, 2 * l2, 2 * l3,
                        2 * m1, 2 * m2, 2 * m3
                    );
                }
            }
        }
    }
}


fn bench_wigner3j(c: &mut Criterion) {
    let mut group = c.benchmark_group("0382/WignerSymbol");
    group.sample_size(10);
    group.sampling_mode(criterion::SamplingMode::Flat);

    unsafe {
        wigner_symbol_0382::ws0382_init(600);
    }

    for &max_angular in &[4, 8, 12, 16, 20] {
        group.bench_function(format!("max_angular={}", max_angular), |b| {
            b.iter(|| compute_all_wigner_3j(max_angular))
        });
    }

    group.bench_function("j = (300, 100, 250)", |b| {
        b.iter(|| compute_large_wigner_3j(300, 100, 250))
    });

    let v = unsafe {
        wigner_symbol_0382::ws0382_wigner_3j(2 * 300, 2 * 100, 2 * 250, 0, 2, -2)
    };
    dbg!(v);
}

criterion_group!(wigner3j, bench_wigner3j);
criterion_main!(wigner3j);

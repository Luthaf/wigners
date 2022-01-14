#![allow(clippy::needless_return)]

use wigner::wigner_3j;

use criterion::{Criterion, criterion_group, criterion_main};

fn bench_wigner3j(c: &mut Criterion) {
    for &max_angular in &[4, 8, 12, 20] {
        c.bench_function(&format!("wigner max_angular={}", max_angular), |b| {
            b.iter(|| {
                for l1 in 0..=max_angular {
                    for l2 in 0..=max_angular {
                        for l3 in 0..=3 {
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

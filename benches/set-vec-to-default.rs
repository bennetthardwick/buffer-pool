#![feature(test)]

extern crate test;
use bufferpool::*;
use test::Bencher;

const MAX: usize = 4096;

#[bench]
fn bench_default_buffer_pool_vec(b: &mut Bencher) {
    let mut pool: BufferPool<usize> = BufferPoolBuilder::new()
        .with_buffer_size(MAX)
        .with_capacity(MAX)
        .build();

    let mut data: Vec<BufferPoolReference<usize>> = Vec::with_capacity(MAX);

    for _ in 0..MAX {
        data.push(pool.get_space().unwrap());
    }

    b.iter(|| {
        pool.clear();
    });
}

#[bench]
fn bench_default_vec_of_vecs(b: &mut Bencher) {
    let mut data = vec![vec![0 as usize; MAX]; MAX];

    b.iter(|| {
        for value in data.iter_mut() {
            for value in value.iter_mut() {
                *value = 0;
            }
        }
    });
}

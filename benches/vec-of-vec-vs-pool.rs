#![feature(test)]

extern crate test;
use buffer_pool::*;
use test::Bencher;

const MAX: usize = 4096;

#[bench]
fn bench_buffer_pool_vec(b: &mut Bencher) {
    b.iter(|| {
        let mut pool: BufferPool<usize> = BufferPoolBuilder::new()
            .with_buffer_size(MAX)
            .with_capacity(MAX)
            .build();

        let mut data: Vec<BufferPoolReference<'_, usize>> = Vec::with_capacity(MAX);

        for _ in 0..MAX {
            data.push(pool.get_space().unwrap());
        }

        for (index, value) in data.iter_mut().enumerate() {
            for (inner_index, value) in value.as_mut().iter_mut().enumerate() {
                *value = index + inner_index;
            }
        }

        let data = data
            .iter()
            .map(|x| x.as_ref().iter().fold(0, |a, b| (a + b) / 2))
            .fold(0, |a, b| (a + b) / 2);

        assert_eq!(data, MAX * 2 - 6);
    });
}

#[bench]
fn bench_vec_of_vecs(b: &mut Bencher) {
    b.iter(|| {
        let mut data = vec![vec![0 as usize; MAX]; MAX];

        for (index, value) in data.iter_mut().enumerate() {
            for (inner_index, value) in value.iter_mut().enumerate() {
                *value = index + inner_index;
            }
        }

        let data = data
            .iter()
            .map(|x| x.iter().fold(0, |a, b| (a + b) / 2))
            .fold(0, |a, b| (a + b) / 2);

        assert_eq!(data, MAX * 2 - 6);
    });
}

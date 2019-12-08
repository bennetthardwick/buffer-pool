#![feature(test)]

extern crate test;
use bufferpool::*;
use test::Bencher;

const MAX: usize = 4096;

#[bench]
fn bench_iter_buffer_pool(b: &mut Bencher) {
    let mut pool: BufferPool<usize> = BufferPoolBuilder::new()
        .with_buffer_size(MAX)
        .with_capacity(MAX)
        .build();

    let mut data: Vec<BufferPoolReference<'_, usize>> = Vec::with_capacity(MAX);

    for _ in 0..MAX {
        data.push(pool.get_space().unwrap());
    }

    b.iter(|| {
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
fn bench_iter_vec_of_vecs(b: &mut Bencher) {
    let mut data = vec![vec![0 as usize; MAX]; MAX];

    b.iter(|| {
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

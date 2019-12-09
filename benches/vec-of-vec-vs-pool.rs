#![feature(test)]

extern crate test;
use bufferpool::*;
use test::Bencher;

const COUNT: usize = 4096;
const BUFFER_SIZE: usize = 4096;

#[bench]
fn bench_iter_buffer_pool(b: &mut Bencher) {
    let mut pool: BufferPool<usize> = BufferPoolBuilder::new()
        .with_buffer_size(BUFFER_SIZE)
        .with_capacity(COUNT)
        .build();

    let mut data: Vec<BufferPoolReference<usize>> = Vec::with_capacity(COUNT);

    for _ in 0..COUNT {
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

        assert_eq!(data, (COUNT + BUFFER_SIZE) - 6);
    });
}

#[bench]
fn bench_iter_vec_of_vecs(b: &mut Bencher) {
    let mut data = vec![];

    for _ in 0..COUNT {
        data.push(vec![0 as usize; BUFFER_SIZE]);
    }

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

        assert_eq!(data, (COUNT + BUFFER_SIZE) - 6);
    });
}

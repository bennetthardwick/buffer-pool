#![feature(test)]

extern crate test;
use bufferpool::*;
use test::Bencher;

const MAX: usize = 4096;

#[bench]
fn bench_ownership_buffer_pool_vec(b: &mut Bencher) {
    let mut pool: BufferPool<usize> = BufferPoolBuilder::new()
        .with_buffer_size(MAX)
        .with_capacity(MAX)
        .build();

    let mut temp: Vec<BufferPoolReference<'_, usize>> = Vec::with_capacity(MAX);

    b.iter(|| {
        for index in 0..MAX {
            let mut buffer = pool.get_space().unwrap();

            for (inner_index, value) in buffer.as_mut().iter_mut().enumerate() {
                *value = index + inner_index;
            }

            temp.push(buffer);
        }

        let value = temp
            .iter()
            .map(|x| x.as_ref().iter().fold(0, |a, b| (a + b) / 2))
            .fold(0, |a, b| (a + b) / 2);

        temp.drain(..).for_each(|x| drop(x));

        assert_eq!(value, MAX * 2 - 6);
    });
}

#[bench]
fn bench_ownership_vec_of_vecs(b: &mut Bencher) {
    let mut data = vec![vec![0 as usize; MAX]; MAX];
    let mut temp = Vec::with_capacity(MAX);

    b.iter(|| {
        for index in 0..MAX {
            let mut buffer = data.pop().unwrap();

            for (inner_index, value) in buffer.iter_mut().enumerate() {
                *value = index + inner_index;
            }

            temp.push(buffer);
        }

        let value = temp
            .iter()
            .map(|x| x.iter().fold(0, |a, b| (a + b) / 2))
            .fold(0, |a, b| (a + b) / 2);

        temp.drain(..).for_each(|x| data.push(x));

        assert_eq!(value, MAX * 2 - 6);
    });
}

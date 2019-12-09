#![feature(test)]

extern crate test;
use bufferpool::*;
use test::Bencher;

const MAX: usize = 4096;
const MIN: usize = 1024;

#[bench]
fn bench_resize_buffer_pool_vec(b: &mut Bencher) {
    b.iter(|| {
        let mut pool: BufferPool<usize> = BufferPoolBuilder::new()
            .with_buffer_size(MIN)
            .with_capacity(MIN)
            .build();

        let mut temp: Vec<BufferPoolReference<usize>> = Vec::with_capacity(MAX);

        for _ in 0..MIN {
            temp.push(pool.get_space().unwrap());
        }

        temp.iter_mut()
            .enumerate()
            .for_each(|(outer_index, inner_array)| {
                inner_array
                    .as_mut()
                    .iter_mut()
                    .enumerate()
                    .for_each(|(inner_index, value)| *value = inner_index + outer_index)
            });

        let value = {
            temp.drain(..)
                .map(|x| {
                    let y = x.as_ref().iter().fold(0, |a, b| (a + b) / 2);
                    drop(x);
                    y
                })
                .fold(0, |a, b| (a + b) / 2)
        };

        assert_eq!(value, (MIN * 2) - 6);

        pool.resize_len_and_buffer(MAX, MAX);

        for _ in 0..MAX {
            temp.push(pool.get_space().unwrap());
        }

        temp.iter_mut()
            .enumerate()
            .for_each(|(outer_index, inner_array)| {
                inner_array
                    .as_mut()
                    .iter_mut()
                    .enumerate()
                    .for_each(|(inner_index, value)| *value = inner_index + outer_index)
            });

        let value = {
            temp.drain(..)
                .map(|x| {
                    let y = x.as_ref().iter().fold(0, |a, b| (a + b) / 2);
                    drop(x);
                    y
                })
                .fold(0, |a, b| (a + b) / 2)
        };

        assert_eq!(value, (MAX * 2) - 6);
    });
}

#[bench]
fn bench_resize_vec_of_vecs(b: &mut Bencher) {
    b.iter(|| {
        let mut data = vec![vec![0 as usize; MIN]; MIN];

        let mut temp: Vec<Vec<usize>> = Vec::with_capacity(MAX);

        for _ in 0..MIN {
            temp.push(data.pop().unwrap());
        }

        temp.iter_mut()
            .enumerate()
            .for_each(|(outer_index, inner_array)| {
                inner_array
                    .iter_mut()
                    .enumerate()
                    .for_each(|(inner_index, value)| *value = inner_index + outer_index)
            });

        let value = temp
            .drain(..)
            .map(|x| x.iter().fold(0, |a, b| (a + b) / 2))
            .fold(0, |a, b| (a + b) / 2);

        assert_eq!(value, (MIN * 2) - 6);

        data.iter_mut().for_each(|array| array.resize(MAX, 0));
        data.resize(MAX, vec![0 as usize; MAX]);

        for _ in 0..MAX {
            temp.push(data.pop().unwrap());
        }

        temp.iter_mut()
            .enumerate()
            .for_each(|(outer_index, inner_array)| {
                inner_array
                    .iter_mut()
                    .enumerate()
                    .for_each(|(inner_index, value)| *value = inner_index + outer_index)
            });

        let value = temp
            .drain(..)
            .map(|x| x.iter().fold(0, |a, b| (a + b) / 2))
            .fold(0, |a, b| (a + b) / 2);

        assert_eq!(value, (MAX * 2) - 6);
    });
}

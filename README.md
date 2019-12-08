# buffer-pool

[![Build Status](https://travis-ci.org/bennetthardwick/buffer-pool.svg?branch=master)](https://travis-ci.org/bennetthardwick/buffer-pool)

A Rust "vector of vectors" backed by one contiguous vector. Allows mutable borrows of non-overlapping buffers.

## What is BufferPool

`BufferPool` is a crate that lets you get a slice of a certain size without having to allocate space for a new vector.
It does this by pre-allocating a certain region of memory and then handing out references to slices inside this region. It's useful in applications such as audio programming, where allocating and freeing data can be expensive.

Since it's backed by a single region in memory, it benefits from less cache misses than other representations (such as `Vec<Vec<T>>`.

## Usage

When creating a new `BufferPool`, it's recommended to use the `BufferPoolBuilder` interface.
While it's completely possible to create a `BufferPool` yourself, it's less efficient if you know the size of the pool in advance.

```rust
let mut pool: BufferPool<usize> = BufferPoolBuilder::new()
    .with_buffer_size(1024)
    .with_capacity(100)
    .build();

let mut buffer = pool.get_cleared_space().unwrap();

for (index, value) in buffer.as_mut().iter_mut().enumerate() {
    *value = index;
}

let sum: usize = buffer.as_ref().iter().sum();

println!("Sum {}", sum);
```

## Benchmarks

While the main point of this crate is to be able to mutably borrow several members of a vector,
the fact that it's backed by a single contiguous region also has positive performance implications.

A benchmark of `Vec<Vec<usize>>` vs `BufferPool<usize>` with 4096 buffers of size 4096 shows that `BufferPool<usize>` is actually faster.

```
running 2 tests
test bench_buffer_pool_vec ... bench:  54,792,501 ns/iter (+/- 1,715,295)
test bench_vec_of_vecs     ... bench:  71,613,118 ns/iter (+/- 3,009,026)
```

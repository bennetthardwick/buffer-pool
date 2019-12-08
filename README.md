# buffer-pool

[![Build Status](https://travis-ci.org/bennetthardwick/buffer-pool.svg?branch=master)](https://travis-ci.org/bennetthardwick/buffer-pool)

A Rust "vector of vectors" backed by one contiguous vector. Allows mutable borrows of non-overlapping buffers.

## What is BufferPool

`BufferPool` is a crate that lets you get a slice of a certain size without having to allocate space for a new vector.
It does this by pre-allocating a certain region of memory and then handing out references to slices inside this region. It's useful in applications such as audio programming, where allocating and freeing data can be expensive.

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

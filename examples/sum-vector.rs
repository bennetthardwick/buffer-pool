use bufferpool::{BufferPool, BufferPoolBuilder};

fn main() {
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
}

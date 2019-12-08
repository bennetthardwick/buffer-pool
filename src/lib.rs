extern crate alloc;

use alloc::rc::Rc;
use core::cell::RefCell;
use core::marker::PhantomData;

type Used = Rc<RefCell<Vec<u32>>>;

const BITS_IN_U32: usize = 32;

fn value_of_index(values: &[u32], index: usize) -> Result<bool, ()> {
    let value_index = index / BITS_IN_U32;
    let offset = index % BITS_IN_U32;

    if let Some(v) = values.get(value_index) {
        if offset < 32 {
            Ok(v & (1 << offset) != 0)
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}

fn update_index(values: &mut [u32], index: usize, value: bool) -> Result<(), ()> {
    let value_index = index / BITS_IN_U32;
    let offset = index % BITS_IN_U32;

    if let Some(v) = values.get_mut(value_index) {
        if offset < 32 {
            let mask = 1 << offset;
            if value {
                *v |= mask;
            } else {
                *v &= !mask;
            }
            Ok(())
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}

pub struct BufferPool<V: Default + Clone> {
    buffer: Vec<V>,
    buffer_size: usize,
    used: Used,
}

pub struct BufferPoolBuilder<V: Default + Clone> {
    buffer_size: usize,
    capacity: usize,
    marker: PhantomData<V>,
}

impl<V: Clone + Default> Default for BufferPoolBuilder<V> {
    fn default() -> BufferPoolBuilder<V> {
        BufferPoolBuilder {
            buffer_size: 1024,
            capacity: 0,
            marker: PhantomData {},
        }
    }
}

impl<V: Default + Clone> BufferPoolBuilder<V> {
    pub fn new() -> BufferPoolBuilder<V> {
        BufferPoolBuilder::default()
    }

    pub fn with_capacity(mut self, capacity: usize) -> BufferPoolBuilder<V> {
        self.capacity = capacity;
        self
    }

    pub fn with_buffer_size(mut self, buffer_size: usize) -> BufferPoolBuilder<V> {
        self.buffer_size = buffer_size;
        self
    }

    pub fn build(self) -> BufferPool<V> {
        BufferPool {
            buffer_size: self.buffer_size,
            buffer: vec![V::default(); self.capacity * self.buffer_size],
            used: Rc::new(RefCell::new(vec![
                0;
                if self.capacity == 0 {
                    0
                } else {
                    1 + ((self.capacity - 1) / BITS_IN_U32)
                }
            ])),
        }
    }
}

impl<V: Default + Clone> Default for BufferPool<V> {
    fn default() -> BufferPool<V> {
        BufferPoolBuilder::default().build()
    }
}

impl<V: Default + Clone> BufferPool<V> {
    pub fn builder() -> BufferPoolBuilder<V> {
        BufferPoolBuilder::default()
    }

    fn find_free_index(&self) -> Result<usize, ()> {
        let mut index = 0;
        let max_index = self.len();

        loop {
            let used = self.used.borrow();
            let used = used.as_slice();

            if index % BITS_IN_U32 == 0 {
                if let Some(value) = used.get(index / BITS_IN_U32) {
                    if value == &core::u32::MAX {
                        index += BITS_IN_U32;
                        continue;
                    }
                }
            }

            if let Ok(value) = value_of_index(used, index) {
                if !value {
                    return Ok(index);
                } else {
                    index += 1;

                    if max_index <= index {
                        return Err(());
                    }
                }
            } else {
                return Err(());
            }
        }
    }

    pub fn clear(&mut self) {
        for value in self.buffer.iter_mut() {
            *value = V::default();
        }
    }

    fn set_index_used(&mut self, index: usize) -> Result<(), ()> {
        let mut used = self.used.borrow_mut();
        let used = used.as_mut_slice();
        update_index(used, index, true)
    }

    fn find_free_index_and_use(&mut self) -> Result<usize, ()> {
        if let Ok(index) = self.find_free_index() {
            self.set_index_used(index).map(|_| index)
        } else {
            Err(())
        }
    }

    pub fn capacity(&self) -> usize {
        self.buffer.len() / self.buffer_size
    }

    pub fn change_buffer_size(&mut self, new_buffer_size: usize) {
        self.buffer_size = new_buffer_size;
        self.resize(self.len());
    }

    pub fn try_change_buffer_size(&mut self, new_buffer_size: usize) -> Result<(), ()> {
        self.buffer_size = new_buffer_size;
        self.try_resize(self.len())
    }

    pub fn len(&self) -> usize {
        self.buffer.len() / self.buffer_size
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn reserve(&mut self, additional: usize) {
        self.resize(self.len() + additional);
    }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), ()> {
        self.try_resize(self.len() + additional)
    }

    pub fn is_borrowed(&self) -> bool {
        let mut index = 0;
        let max_index = self.len();

        let used = self.used.borrow();
        let used = used.as_slice();

        loop {
            if let Ok(value) = value_of_index(used, index) {
                if value {
                    return false;
                } else {
                    index += 1;

                    if max_index <= index {
                        break;
                    }
                }
            } else {
                return false;
            }
        }

        false
    }

    pub fn resize(&mut self, new_len: usize) {
        if let Err(()) = self.try_resize(new_len) {
            panic!("Can't resize when borrowed!");
        }
    }

    pub fn try_resize(&mut self, new_len: usize) -> Result<(), ()> {
        if self.is_borrowed() {
            Err(())
        } else {
            self.buffer
                .resize_with(new_len * self.buffer_size, V::default);

            let mut used_capacity = self.used.borrow().len() * BITS_IN_U32;

            while used_capacity < new_len {
                let new_len = self.used.borrow().len() + 1;

                self.used.borrow_mut().resize(new_len, 0);

                used_capacity = self.used.borrow().len() * BITS_IN_U32;
            }

            Ok(())
        }
    }

    pub fn get_cleared_space<'a, 'b>(&'a mut self) -> Result<BufferPoolReference<'b, V>, ()> {
        self.get_space().and_then(|mut space| {
            for value in space.as_mut().iter_mut() {
                *value = V::default();
            }

            Ok(space)
        })
    }

    pub fn get_space<'a, 'b>(&'a mut self) -> Result<BufferPoolReference<'b, V>, ()> {
        self.find_free_index_and_use().and_then(|index| {
            let slice = unsafe {
                alloc::slice::from_raw_parts_mut(
                    self.buffer.as_mut_ptr().add(index * self.buffer_size),
                    self.buffer_size,
                )
            };

            Ok(BufferPoolReference {
                index,
                used: Rc::clone(&self.used),
                slice,
            })
        })
    }
}

pub struct BufferPoolReference<'a, V> {
    index: usize,
    used: Used,
    slice: &'a mut [V],
}

impl<V> AsMut<[V]> for BufferPoolReference<'_, V> {
    fn as_mut(&mut self) -> &mut [V] {
        self.slice
    }
}

impl<V> AsRef<[V]> for BufferPoolReference<'_, V> {
    fn as_ref(&self) -> &[V] {
        self.slice
    }
}

impl<V> Drop for BufferPoolReference<'_, V> {
    fn drop(&mut self) {
        let mut used = self.used.borrow_mut();
        let used = used.as_mut_slice();

        if update_index(used, self.index, false).is_err() {
            panic!("Unable to free reference for index {}!", self.index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_add_capacity() {
        let mut pool: BufferPool<f32> = BufferPool::default();

        assert_eq!(pool.capacity(), 0);

        pool.reserve(1);

        assert_eq!(pool.capacity(), 1);

        pool.reserve(1);

        assert_eq!(pool.capacity(), 2);
    }

    #[test]
    fn it_should_get_space_if_capacity() {
        let mut pool: BufferPool<f32> = BufferPool::default();

        assert_eq!(pool.capacity(), 0);

        assert!(pool.get_space().is_err());

        pool.resize(1);

        let index = pool.get_space().unwrap();

        assert!(pool.get_space().is_err());
        assert_eq!(index.index, 0);
    }

    #[test]
    fn it_should_work_with_interesting_sizes() {
        let sizes: Vec<usize> = vec![12, 100, 1001, 1024, 2048, 4096, 1];

        for buffer_size in sizes.iter() {
            for capacity in sizes.iter() {
                let mut pool: BufferPool<f32> = BufferPoolBuilder::new()
                    .with_buffer_size(*buffer_size)
                    .with_capacity(*capacity)
                    .build();

                assert_eq!(pool.capacity(), *capacity);
                assert_eq!(pool.get_space().is_err(), false);
            }
        }
    }

    #[test]
    fn it_should_return_space_when_deallocated() {
        let mut pool: BufferPool<f32> = BufferPool::default();

        assert_eq!(pool.capacity(), 0);
        pool.reserve(1);

        {
            let index = pool.get_space().unwrap();
            assert!(pool.get_space().is_err());
            assert_eq!(index.index, 0);
        }

        assert!(pool.get_space().is_ok());
    }

    #[test]
    fn it_should_update_internal_buffer() {
        let buffer_size = 10;
        let mut pool: BufferPool<f32> = BufferPool::default();
        pool.change_buffer_size(buffer_size);
        pool.reserve(10);

        let mut a = pool.get_space().unwrap();
        let mut b = pool.get_space().unwrap();

        for value in a.as_mut().iter_mut() {
            *value = 1.;
        }

        for value in b.as_mut().iter_mut() {
            *value = 2.;
        }

        assert_eq!(
            pool.buffer[0..(buffer_size)],
            vec![1. as f32; buffer_size][..]
        );

        assert_eq!(*a.as_ref(), vec![1. as f32; buffer_size][..]);

        assert_eq!(
            pool.buffer[(buffer_size)..(2 * buffer_size)],
            vec![2. as f32; buffer_size][..]
        );

        assert_eq!(*b.as_ref(), vec![2. as f32; buffer_size][..]);
    }

    #[test]
    fn it_should_not_default_space_when_deallocated() {
        let buffer_size = 10;
        let mut pool: BufferPool<f32> = BufferPool::default();
        pool.change_buffer_size(buffer_size);
        pool.reserve(10);

        {
            let mut a = pool.get_space().unwrap();

            for value in a.as_mut().iter_mut() {
                *value = 1.;
            }

            assert_eq!(
                pool.buffer[0..(buffer_size)],
                vec![1. as f32; buffer_size][..]
            );

            assert_eq!(*a.as_ref(), vec![1. as f32; buffer_size][..]);
        }

        assert_eq!(
            pool.buffer[0..(buffer_size)],
            vec![1. as f32; buffer_size][..]
        );
    }

    #[test]
    fn it_should_clear_space_if_explicitly_requested() {
        let buffer_size = 10;
        let mut pool: BufferPool<f32> = BufferPool::default();
        pool.change_buffer_size(buffer_size);
        pool.reserve(10);

        {
            let mut a = pool.get_space().unwrap();

            for value in a.as_mut().iter_mut() {
                *value = 1.;
            }

            assert_eq!(
                pool.buffer[0..(buffer_size)],
                vec![1. as f32; buffer_size][..]
            );

            assert_eq!(*a.as_ref(), vec![1. as f32; buffer_size][..]);
        }

        let space = pool.get_cleared_space().unwrap();

        assert_eq!(
            pool.buffer[0..(buffer_size)],
            vec![0. as f32; buffer_size][..]
        );

        assert_eq!(*space.as_ref(), vec![0. as f32; buffer_size][..]);
    }
}

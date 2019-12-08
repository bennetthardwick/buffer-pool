extern crate bitvec;
extern crate log;

use bitvec::vec::BitVec;
use core::cell::RefCell;
use std::rc::Rc;

type Used = Rc<RefCell<BitVec>>;

pub struct BufferPool<V: Default> {
    buffer: Vec<V>,
    buffer_size: usize,
    used: Used,
}

impl<V: Default> BufferPool<V> {
    pub fn new() -> BufferPool<V> {
        BufferPool {
            buffer: vec![],
            buffer_size: 1024,
            used: Rc::new(RefCell::new(BitVec::new())),
        }
    }

    fn find_free_index(&self) -> Result<usize, ()> {
        let used = self.used.borrow();
        if let Some(index) = used.iter().position(|&x| x == false) {
            Ok(index)
        } else {
            Err(())
        }
    }

    fn set_index_used(&mut self, index: usize) {
        let mut used = self.used.borrow_mut();
        used.set(index, true);
    }

    fn find_free_index_and_use(&mut self) -> Result<usize, ()> {
        if let Ok(index) = self.find_free_index() {
            self.set_index_used(index);
            Ok(index)
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

    pub fn len(&self) -> usize {
        self.buffer.len() / self.buffer_size
    }

    pub fn reserve(&mut self, additional: usize) {
        self.resize(self.len() + additional);
    }

    pub fn is_borrowed(&self) -> bool {
        let used = self.used.borrow();
        if let Some(_) = used.iter().position(|&x| x == true) {
            true
        } else {
            false
        }
    }

    pub fn resize(&mut self, new_len: usize) {
        if self.is_borrowed() {
            panic!("Can't resize when borrowed!");
        }

        self.buffer
            .resize_with(new_len * self.buffer_size, || V::default());

        let mut used = self.used.borrow_mut();
        used.resize_with(new_len, || false);
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
                std::slice::from_raw_parts_mut(
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
        used.set(self.index, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_add_capacity() {
        let mut pool: BufferPool<f32> = BufferPool::new();

        assert_eq!(pool.capacity(), 0);

        pool.reserve(1);

        assert_eq!(pool.capacity(), 1);

        pool.reserve(1);

        assert_eq!(pool.capacity(), 2);
    }

    #[test]
    fn it_should_get_space_if_capacity() {
        let mut pool: BufferPool<f32> = BufferPool::new();

        assert_eq!(pool.capacity(), 0);

        assert!(pool.get_space().is_err());

        pool.resize(1);

        let index = pool.get_space().unwrap();

        assert!(pool.get_space().is_err());
        assert_eq!(index.index, 0);
    }

    #[test]
    fn it_should_return_space_when_deallocated() {
        let mut pool: BufferPool<f32> = BufferPool::new();

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
        let mut pool: BufferPool<f32> = BufferPool::new();
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
        let mut pool: BufferPool<f32> = BufferPool::new();
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
        let mut pool: BufferPool<f32> = BufferPool::new();
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

use parse::arena::ArenaMem;
use std::ops::Add;
use num::traits::Unsigned;
use num::traits::{FromPrimitive, ToPrimitive, One};
use core::slice::{from_raw_parts_mut, from_raw_parts};
use std::marker::PhantomData;
use std::mem;

#[derive(PartialEq,Debug,Clone,Copy)]
pub struct Vector<T, I> {
    // vector stored in arena
    pub first: I, // first element of vector
    pub len: usize, // element count
    pub phantom: PhantomData<*const T>,
}

impl<T, I: Unsigned + Add + FromPrimitive + ToPrimitive + Copy> Vector<T, I> {
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn set_len(&mut self, l: usize) {
        debug_assert!(l <= self.len);
        self.len = l;
    }

    pub fn as_slice_mut<'a, A>(&self, arena: &'a mut ArenaMem<A, I>) -> &'a mut [T] {
        if self.len == 0 {
            Default::default() // empty vector => empty slice
        } else {
            unsafe { from_raw_parts_mut(arena.deref_as_mut::<T>(self.first), self.len) }
        }
    }

    pub fn as_slice<'a, A>(&self, arena: &'a ArenaMem<A, I>) -> &'a [T] {
        if self.len == 0 {
            Default::default() // empty vector => empty slice
        } else {
            unsafe { from_raw_parts(arena.deref_as::<T>(self.first), self.len) }
        }
    }

    #[inline(always)]
    pub fn to_id(&self, ix: usize) -> I {
        debug_assert!((ix < self.len), "Index error");
        self.first + FromPrimitive::from_usize(ix).unwrap()
    }

    #[inline(always)]
    pub fn set<'a, A>(&self, ix: usize, arena: &'a mut ArenaMem<A, I>, v: T) {
        debug_assert!((ix < self.len), "Index error");
        mem::replace(arena.deref_as_offset_mut::<T>(self.first, ix), v);
    }

    #[inline(always)]
    pub fn get<'a, A>(&self, ix: usize, arena: &'a ArenaMem<A, I>) -> &'a T {
        debug_assert!((ix < self.len), "Index error");
        arena.deref_as_offset::<T>(self.first, ix)
    }

    pub fn iter<'a, A>(&self, arena: &'a ArenaMem<A, I>) -> VectorArenaIter<'a, T, A, I> {
        VectorArenaIter::<T, A, I> {
            arena: arena,
            first: self.first,
            curr: 0,
            len: self.len,
            phantom: PhantomData,
        }
    }

    pub fn from_iter<It: IntoIterator<Item = T>, A>(&self, iter: It, arena: &mut ArenaMem<A, I>) {
        // saves iterator values into self vector, no reallocations expected!!!
        //
        // generic/slow version. No vectorization is done...
        // let mut ix = 0;
        // for v in iter {
        // self.set(ix, arena, v);
        // ix += 1;
        // }
        //


        // more compiler-friendly/autovectorization version
        let mut ptr = arena.to_ptr::<T>(self.first);
        for v in iter {
            unsafe {
                *ptr = v;
                ptr = ptr.offset(1);
            }
        }
    }

    pub fn id_iter(&self) -> VectItemIter<T, I> {
        VectItemIter::<T, I> {
            curr: self.first,
            last: self.first + FromPrimitive::from_usize(self.len).unwrap() - One::one(),
            phantom: PhantomData,
        }
    }
}

#[derive(Debug,Clone,Copy)]
pub struct VectorArenaIter<'a, T: 'a, A: 'a, I: 'a>
    where I: Unsigned + ToPrimitive
{
    // general purpose safe (non reference-saving) iterator
    arena: &'a ArenaMem<A, I>,
    first: I,
    curr: usize, // current index vector
    len: usize, // total length of vector
    phantom: PhantomData<*const T>,
}

impl<'a, T: 'a, A: 'a, I:Unsigned+ToPrimitive+FromPrimitive+Copy> Iterator for VectorArenaIter<'a, T, A, I> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        if self.curr < self.len {
            let r = Some(self.arena.deref_as_offset::<T>(self.first, self.curr));
            self.curr += 1;
            r
        } else {
            None
        }
    }
}

#[derive(PartialEq,Debug,Copy,Clone)]
pub struct VectItemIter<T, I> {
    // general purpose safe (non reference-saving) iterator over vector element ids
    // warning!!! meant to iterate only over vector elements having the same type as arena!!!
    // e.g. ArenaMem<AST> -> VectItemIter<AST>
    curr: I,
    last: I,
    phantom: PhantomData<*const T>,
}

impl<T, I: Unsigned + Add + PartialOrd + Copy> Iterator for VectItemIter<T, I> {
    type Item = I;
    fn next(&mut self) -> Option<I> {
        if self.curr <= self.last {
            let r = Some(self.curr);
            self.curr = self.curr + One::one();
            r
        } else {
            None
        }
    }
}

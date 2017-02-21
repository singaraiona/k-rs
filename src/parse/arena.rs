use alloc::raw_vec::RawVec;
use std::mem::size_of;
use std::marker::PhantomData;
use core::ops::{Index, IndexMut};
use std::mem::transmute;
use std::mem::replace;
use std::default::Default;
use core::slice::{from_raw_parts_mut, from_raw_parts};
use core::iter::IntoIterator;
use std::fmt;
use std::ops::Add;
use num::traits::Unsigned;
use num::traits::{FromPrimitive, ToPrimitive, One, Zero};

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Vect<T, I> {
    // vector stored in arena
    first: I, // first element of vector
    len: usize, // element count
    phantom: PhantomData<*const T>,
}

impl<T, I: Unsigned + Add + FromPrimitive + ToPrimitive + Copy> Vect<T, I> {
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn set_len(&mut self, l: usize) {
        debug_assert!(l <= self.len);
        self.len = l;
    }

    pub fn as_slice_mut<A>(&self, arena: &mut ArenaMem<A, I>) -> &mut [T] {
        if self.len == 0 {
            Default::default() // empty vector => empty slice
        } else {
            unsafe { from_raw_parts_mut(arena.deref_as_mut::<T>(self.first), self.len) }
        }
    }

    pub fn as_slice<A>(&self, arena: &ArenaMem<A, I>) -> &[T] {
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
    pub fn set<A>(&self, ix: usize, arena: &mut ArenaMem<A, I>, v: T) {
        debug_assert!((ix < self.len), "Index error");
        replace(arena.deref_as_offset_mut::<T>(self.first, ix), v);

    }

    #[inline(always)]
    pub fn get<'a, A>(&self, ix: usize, arena: &'a ArenaMem<A, I>) -> &'a T {
        debug_assert!((ix < self.len), "Index error");
        arena.deref_as_offset::<T>(self.first, ix)
    }

    pub fn iter<'a, A>(&self, arena: &'a ArenaMem<A, I>) -> VectArenaIter<'a, T, A, I> {
        VectArenaIter::<T, A, I> {
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
pub struct VectArenaIter<'a, T: 'a, A: 'a, I: 'a>
    where I: Unsigned + ToPrimitive
{
    // general purpose safe (non reference-saving) iterator
    arena: &'a ArenaMem<A, I>,
    first: I,
    curr: usize, // current index vector
    len: usize, // total length of vector
    phantom: PhantomData<*const T>,
}

impl<'a, T: 'a, A: 'a, I:Unsigned+ToPrimitive+FromPrimitive+Copy> Iterator for VectArenaIter<'a, T, A, I> {
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

pub struct ArenaMem<T, I> {
    mem: RawVec<u8>,
    next: I,
    phantom: PhantomData<*const T>,
}

fn align(s: usize, a: usize) -> usize {
    // align s to a
    ((s + a - 1) / a) * a
}

impl<T, I: Unsigned + ToPrimitive> fmt::Debug for ArenaMem<T, I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "ArenaMem. next {}",
               ToPrimitive::to_u64(&self.next).unwrap())
    }
}

impl<T, I: Unsigned + Zero + ToPrimitive + FromPrimitive + Copy> ArenaMem<T, I> {
    #[inline]
    pub fn new() -> ArenaMem<T, I> {
        ArenaMem::<T, I> {
            mem: RawVec::new(),
            next: Zero::zero(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> ArenaMem<T, I> {
        let mut m = RawVec::with_capacity(capacity * size_of::<T>());
        m.reserve(0, capacity);

        ArenaMem::<T, I> {
            mem: m,
            next: Zero::zero(),
            phantom: PhantomData,
        }
    }

    #[inline(always)]
    fn offset(&self, item: I) -> isize {
        ToPrimitive::to_isize(&(item * (FromPrimitive::from_usize(size_of::<T>()).unwrap())))
            .unwrap()
    }

    #[inline(always)]
    pub fn deref(&self, item: I) -> &T {
        unsafe { transmute::<*mut u8, &T>(self.mem.ptr().offset(self.offset(item))) }
    }

    #[inline(always)]
    pub fn deref_mut(&self, item: I) -> &mut T {
        unsafe { transmute::<*mut u8, &mut T>(self.mem.ptr().offset(self.offset(item))) }
    }

    #[inline(always)]
    pub fn deref_as<E>(&self, item: I) -> &E {
        unsafe { transmute::<*mut u8, &E>(self.mem.ptr().offset(self.offset(item))) }
    }

    #[inline(always)]
    pub fn deref_as_mut<E>(&mut self, item: I) -> &mut E {
        unsafe { transmute::<*mut u8, &mut E>(self.mem.ptr().offset(self.offset(item))) }
    }

    #[inline(always)]
    pub fn deref_as_offset<E>(&self, item: I, offset: usize) -> &E {
        let off: isize = (offset * size_of::<E>()) as isize;
        unsafe { transmute::<*mut u8, &E>(self.mem.ptr().offset(self.offset(item) + off)) }
    }

    #[inline(always)]
    pub fn deref_as_offset_mut<E>(&self, item: I, offset: usize) -> &mut E {
        let off: isize = (offset * size_of::<E>()) as isize;
        unsafe { transmute::<*mut u8, &mut E>(self.mem.ptr().offset(self.offset(item) + off)) }
    }

    #[inline(always)]
    pub fn to_ptr<E>(&self, item: I) -> *mut E {
        unsafe { transmute::<*mut u8, *mut E>(self.mem.ptr().offset(self.offset(item))) }
    }

    #[inline(always)]
    pub fn push(&mut self, data: T) -> I {
        // increase arena size if necessary
        let used = self.offset(self.next) as usize;
        self.mem.reserve(used, size_of::<T>());

        replace(self.deref_mut(self.next), data);
        let r = self.next;
        self.next = self.next + One::one();
        r
    }

    #[inline]
    pub fn clear(&mut self) {
        self.next = Zero::zero();
    }

    #[inline]
    pub fn curr(&self) -> I {
        // return current/not-yet-allocated id
        self.next
    }

    #[inline]
    pub fn len(&self) -> usize {
        ToPrimitive::to_usize(&self.next).unwrap()
    }

    #[inline]
    pub fn set_curr(&mut self, curr: I) {
        self.next = curr;
    }

    fn inc_by(&mut self, size: usize) -> I {
        // increase "next" by "size" bytes
        let a = size_of::<T>();
        let r = self.next;
        self.next = self.next +
                    FromPrimitive::from_u64((((size + a - 1) as u64) / a as u64)).unwrap();
        r
    }

    // pub fn push_vec<E>(&mut self, el_count: usize) -> (&mut Vect<E, I>, I) {
    //     // create vector in arena with el_count elements
    //     // differs from alloc_vec in that it saves vec head immediately in arena right before vector elements

    //     // align vector head and contents to T size => allows to use I type for references
    //     let a = size_of::<T>();
    //     let vs = align(size_of::<Vect<E, I>>(), a);
    //     let es = align(size_of::<E>() * el_count, a);
    //     let size = vs + es;

    //     let used = self.offset(self.next) as usize;
    //     // increase arena size if necessary
    //     self.mem.reserve(used, size);

    //     // put vector head first
    //     let mut v = Vect::<E, I> {
    //         first: Zero::zero(),
    //         len: 0,
    //         phantom: PhantomData,
    //     };
    //     let (s1, s2) = split(self);
    //     let iv = s1.inc_by(vs); // vector head index
    //     let av = s1.deref_as_mut::<Vect<E, I>>(iv);
    //     replace(av, v);

    //     if el_count > 0 {
    //         // allocate vector elements and save in vector head
    //         av.first = s2.inc_by(es);
    //         av.len = el_count;
    //     }
    //     (av, iv)
    // }

    pub fn alloc_vec<E>(&mut self, el_count: usize) -> Vect<E, I> {
        // create vector elements in arena and return vector "head"

        let mut v = Vect::<E, I> {
            first: Zero::zero(),
            len: 0,
            phantom: PhantomData,
        };
        if el_count > 0 {
            // align contents to T size => allows to use I type for references
            let a = size_of::<T>();
            let es = align(size_of::<E>() * el_count, a);

            let used = self.offset(self.next) as usize;
            // increase arena size if necessary
            self.mem.reserve(used, es);

            // allocate vector elements and save in vector head
            v.first = self.inc_by(es);
            v.len = el_count;
        }
        v
    }
}

impl<T, I: Unsigned + ToPrimitive + FromPrimitive + Copy> Index<I> for ArenaMem<T, I> {
    type Output = T;

    #[inline(always)]
    fn index(&self, item: I) -> &T {
        self.deref(item)
    }
}

impl<T, I: Unsigned + ToPrimitive + FromPrimitive + Copy> IndexMut<I> for ArenaMem<T, I> {
    #[inline(always)]
    fn index_mut(&mut self, item: I) -> &mut T {
        self.deref_mut(item)
    }
}

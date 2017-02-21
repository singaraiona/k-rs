use alloc::raw_vec::RawVec;
use std::marker::PhantomData;
use core::ops::{Index, IndexMut};
use std::mem::{self, size_of, transmute};
use std::default::Default;
use core::slice::{from_raw_parts_mut, from_raw_parts};
use core::iter::IntoIterator;
use std::fmt;
use std::ops::Add;
use num::traits::Unsigned;
use num::traits::{FromPrimitive, ToPrimitive, One, Zero};
use parse::vector::Vector;
use handle;

pub struct ArenaMem<T, I> {
    mem: RawVec<u8>,
    next: I,
    phantom: PhantomData<*const T>,
}

impl<T, I> fmt::Debug for ArenaMem<T, I>
    where I: Unsigned + ToPrimitive
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "ArenaMem. next {}",
               ToPrimitive::to_u64(&self.next).unwrap())
    }
}

impl<T, I> ArenaMem<T, I>
    where I: Unsigned + Zero + ToPrimitive + FromPrimitive + Copy
{
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

        mem::replace(self.deref_mut(self.next), data);
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

    pub fn push_vec<E>(&mut self, el_count: usize) -> (&mut Vector<E, I>, I) {
        // create vector in arena with el_count elements
        // differs from alloc_vec in that it saves vec head immediately in arena right before vector elements

        // align vector head and contents to T size => allows to use I type for references
        let a = size_of::<T>();
        let vs = align(size_of::<Vector<E, I>>(), a);
        let es = align(size_of::<E>() * el_count, a);
        let size = vs + es;

        let used = self.offset(self.next) as usize;
        // increase arena size if necessary
        self.mem.reserve(used, size);

        // put vector head first
        let mut v = Vector::<E, I> {
            first: Zero::zero(),
            len: 0,
            phantom: PhantomData,
        };
        let (s1, s2) = handle::split(self);
        let iv = s1.inc_by(vs); // vector head index
        let av = s2.deref_as_mut::<Vector<E, I>>(iv);
        mem::replace(av, v);

        if el_count > 0 {
            // allocate vector elements and save in vector head
            av.first = s1.inc_by(es);
            av.len = el_count;
        }
        (av, iv)
    }

    pub fn alloc_vec<E>(&mut self, el_count: usize) -> Vector<E, I> {
        // create vector elements in arena and return vector "head"
        if el_count == 0 {
            return Vector::<E, I> {
                first: Zero::zero(),
                len: 0,
                phantom: PhantomData,
            };
        }
        // align contents to T size => allows to use I type for references
        let a = size_of::<T>();
        let es = align(size_of::<E>() * el_count, a);

        let used = self.offset(self.next) as usize;
        // increase arena size if necessary
        self.mem.reserve(used, es);

        // allocate vector elements and save in vector head
        Vector::<E, I> {
            first: self.inc_by(es),
            len: el_count,
            phantom: PhantomData,
        }
    }
}

impl<T, I> Index<I> for ArenaMem<T, I>
    where I: Unsigned + ToPrimitive + FromPrimitive + Copy
{
    type Output = T;

    #[inline(always)]
    fn index(&self, item: I) -> &T {
        self.deref(item)
    }
}

impl<T, I> IndexMut<I> for ArenaMem<T, I>
    where I: Unsigned + ToPrimitive + FromPrimitive + Copy
{
    #[inline(always)]
    fn index_mut(&mut self, item: I) -> &mut T {
        self.deref_mut(item)
    }
}

fn align(s: usize, a: usize) -> usize {
    // align s to a
    ((s + a - 1) / a) * a
}
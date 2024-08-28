#![allow(dead_code)]
use std::cell::Cell;
use std::fmt;
use typed_arena::Arena;

type EdgeRef<'m, V, F> = (&'m QuadEdge<'m, V, F>, u8);
type EdgeRefCell<'m, V, F> = Cell<Option<EdgeRef<'m, V, F>>>;
type VertCell<'m, V> = Cell<Option<&'m V>>;
type FaceCell<'m, F> = Cell<Option<&'m F>>;

#[derive(Default)]
pub struct QuadEdge<'m, V, F> {
    next: [EdgeRefCell<'m, V, F>; 4],
    data: (
        VertCell<'m, V>,
        FaceCell<'m, F>,
        VertCell<'m, V>,
        FaceCell<'m, F>,
    ),
}

impl<'m, V, F> QuadEdge<'m, V, F> {
    #[inline]
    fn get_next(&'m self, r: u8) -> EdgeRef<'m, V, F> {
        self.next[r.rem_euclid(4) as usize]
            .get()
            .expect("QuadEdge not initialized!")
    }

    #[inline]
    fn set_next(&'m self, r: u8, val: EdgeRef<'m, V, F>) {
        self.next[r.rem_euclid(4) as usize].set(Some(val));
    }

    #[inline]
    fn rot(&'m self, r: u8) -> EdgeRef<'m, V, F> {
        (self, (r + 1).rem_euclid(4))
    }
}

impl<'m, V, F> PartialEq for QuadEdge<'m, V, F> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl<'m, V, F> fmt::Debug for QuadEdge<'m, V, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QuadEdge({:p})", self)
    }
}

#[derive(Default)]
pub struct Manifold<'m, V, F> {
    quads: Arena<QuadEdge<'m, V, F>>,
    verts: Arena<V>,
    faces: Arena<F>,
}

impl<'m, V: Default + Copy, F: Default + Copy> Manifold<'m, V, F> {
    pub fn make_quad(&'m self) -> &'m QuadEdge<'m, V, F> {
        let q = self.quads.alloc(QuadEdge::default());
        q.set_next(0, (q, 0));
        q.set_next(1, (q, 3));
        q.set_next(2, (q, 2));
        q.set_next(3, (q, 1));
        q
    }

    #[inline]
    fn swap(a: EdgeRef<'m, V, F>, b: EdgeRef<'m, V, F>) {
        let (a_next, i_a_next) = a.0.get_next(a.1);
        let (b_next, i_b_next) = b.0.get_next(b.1);
        let a_next_next = a_next.get_next(i_a_next);
        let b_next_next = b_next.get_next(i_b_next);
        a_next.set_next(i_a_next, b_next_next);
        b_next.set_next(i_b_next, a_next_next);
    }

    pub fn splice(&'m self, a: EdgeRef<'m, V, F>, b: EdgeRef<'m, V, F>) {
        if a.1.rem_euclid(2) != b.1.rem_euclid(2) {
            panic!("Incompatible edge types!")
        }

        // Swap vertices
        Self::swap(a, b);

        // Swap faces
        let (a_next, i_a_next) = a.0.get_next(a.1);
        let alpha = a_next.rot(i_a_next);

        let (b_next, i_b_next) = b.0.get_next(b.1);
        let beta = b_next.rot(i_b_next);

        Self::swap(alpha, beta);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_u8_rem_euclid_4_aka_mod() {
        assert_eq!(0u8.rem_euclid(4), 0);
        assert_eq!(1u8.rem_euclid(4), 1);
        assert_eq!(2u8.rem_euclid(4), 2);
        assert_eq!(3u8.rem_euclid(4), 3);
        assert_eq!(4u8.rem_euclid(4), 0);
        assert_eq!(8u8.rem_euclid(4), 0);
        assert_eq!(10u8.rem_euclid(4), 2);
    }

    #[test]
    fn check_internal_operations() {
        let m: Manifold<(), ()> = Manifold::default();
        let q = m.make_quad();
        assert_eq!(q.get_next(0), (q, 0));
        assert_eq!(q.get_next(1), (q, 3));
        assert_eq!(q.get_next(2), (q, 2));
        assert_eq!(q.get_next(3), (q, 1));
        assert_eq!(q.get_next(4), (q, 0));

        assert_eq!(q.rot(0), (q, 1));
        assert_eq!(q.rot(1), (q, 2));
        assert_eq!(q.rot(2), (q, 3));
        assert_eq!(q.rot(3), (q, 0));
        assert_eq!(q.rot(4), (q, 1));
    }

    #[test]
    fn check_splice_operation() {
        let m: Manifold<(), ()> = Manifold::default();
        let q0 = m.make_quad();
        let q1 = m.make_quad();
        m.splice((q0, 0), (q1, 0));
    }
}

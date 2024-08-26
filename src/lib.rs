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
    fn onext(&'m self, r: u8) -> EdgeRef<'m, V, F> {
        self.next[r.rem_euclid(4) as usize].get().unwrap()
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
        q.next[0].set(Some((q, 0)));
        q.next[1].set(Some((q, 3)));
        q.next[2].set(Some((q, 2)));
        q.next[3].set(Some((q, 1)));
        q
    }

    pub fn splice(&'m self, a: EdgeRef<'m, V, F>, b: EdgeRef<'m, V, F>) {
        if a.1.rem_euclid(2) != b.1.rem_euclid(2) {
            panic!("Incompatible edge types!")
        }

        let (qtmp, itmp) = a.0.onext(a.1);
        let _alpha = qtmp.rot(itmp);

        let (qtmp, itmp) = b.0.onext(b.1);
        let _beta = qtmp.rot(itmp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_u8_rem_euclid_4() {
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
        assert_eq!(q.onext(0), (q, 0));
        assert_eq!(q.onext(1), (q, 3));
        assert_eq!(q.onext(2), (q, 2));
        assert_eq!(q.onext(3), (q, 1));
        assert_eq!(q.onext(4), (q, 0));

        assert_eq!(q.rot(0), (q, 1));
        assert_eq!(q.rot(1), (q, 2));
        assert_eq!(q.rot(2), (q, 3));
        assert_eq!(q.rot(3), (q, 0));
        assert_eq!(q.rot(4), (q, 1));
    }

    #[test]
    fn check_splice_operations() {
        let m: Manifold<(), ()> = Manifold::default();
        let q0 = m.make_quad();
        let q1 = m.make_quad();
        m.splice((q0, 1), (q1, 3));

        // TodDo: Add assert statements
    }
}

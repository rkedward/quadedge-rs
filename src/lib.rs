#![allow(dead_code)]
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    fmt,
    io::{self, BufRead, Read, Write},
    ptr,
};
use typed_arena::Arena;

type VertCell<'m, V> = Cell<Option<&'m V>>;
type FaceCell<'m, F> = Cell<Option<&'m F>>;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Node<'m, V, F>(&'m QuadEdge<'m, V, F>, u8);

impl<'m, V: Copy, F: Copy> Node<'m, V, F> {
    pub fn splice(self, other: Node<'m, V, F>) {
        self.swap(other);
        self.next().rot().swap(other.next().rot());
    }

    #[inline]
    fn swap(self, other: Node<'m, V, F>) {
        let self_next = self.next();
        self.set(other.next());
        other.set(self_next);
    }

    #[inline]
    fn next(self) -> Node<'m, V, F> {
        let Node(q, i) = self;
        q.next(i)
    }

    #[inline]
    fn set(self, node: Node<'m, V, F>) {
        let Node(q, i) = self;
        q.set(i, node);
    }

    #[inline]
    fn rot(self) -> Node<'m, V, F> {
        let Node(q, i) = self;
        q.rot(i)
    }
}

#[derive(Default)]
pub struct QuadEdge<'m, V, F> {
    next: [Cell<Option<Node<'m, V, F>>>; 4],
    data: (
        VertCell<'m, V>,
        FaceCell<'m, F>,
        VertCell<'m, V>,
        FaceCell<'m, F>,
    ),
}

impl<'m, V: Copy, F: Copy> QuadEdge<'m, V, F> {
    #[inline]
    pub fn orig(&'m self) -> Node<'m, V, F> {
        self.ind(2)
    }

    #[inline]
    pub fn dest(&'m self) -> Node<'m, V, F> {
        self.ind(2)
    }

    fn ind(&'m self, i: u8) -> Node<'m, V, F> {
        Node(self, i.rem_euclid(4))
    }

    #[inline]
    fn next(&'m self, i: u8) -> Node<'m, V, F> {
        self.next[i.rem_euclid(4) as usize]
            .get()
            .expect("Node not initialized")
    }

    #[inline]
    fn rot(&'m self, i: u8) -> Node<'m, V, F> {
        Node(self, (i + 1).rem_euclid(4))
    }

    #[inline]
    fn set(&'m self, i: u8, node: Node<'m, V, F>) {
        self.next[i.rem_euclid(4) as usize].set(Some(node));
    }

    #[inline]
    fn set_all(&'m self, vals: &[Node<'m, V, F>; 4]) {
        self.set(0, vals[0]);
        self.set(1, vals[1]);
        self.set(2, vals[2]);
        self.set(3, vals[3]);
    }
}

impl<'m, V, F> PartialEq for QuadEdge<'m, V, F> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
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
    qrefs: RefCell<Vec<&'m QuadEdge<'m, V, F>>>,
    verts: Arena<V>,
    faces: Arena<F>,
}

impl<'m, V: Default + Copy, F: Default + Copy> Manifold<'m, V, F> {
    pub fn make_quad(&'m self) -> &'m QuadEdge<'m, V, F> {
        let q = self.quads.alloc(QuadEdge::default());
        self.qrefs.borrow_mut().push(q);
        q.set_all(&[Node(q, 0), Node(q, 3), Node(q, 2), Node(q, 1)]);
        q
    }

    pub fn export<W: Write>(&self, buf: &mut W) -> io::Result<()> {
        // Map the quad's address to its position.
        let map: HashMap<usize, usize> = self
            .qrefs
            .borrow()
            .iter()
            .enumerate()
            .map(|(ind, &q)| (q as *const _ as usize, ind))
            .collect();

        // Write the records to buf, format: [[100,0],[101,1],null,[101,2]]\n ...
        for (_pos, &q) in self.qrefs.borrow().iter().enumerate() {
            write!(buf, "[")?;
            for k in 0..=3 {
                let node = q.next[k]
                    .get()
                    .map(|Node(q, i)| (map[&(q as *const _ as usize)], i));
                match node {
                    Some((r, i)) => write!(buf, "[{},{}]", r, i)?,
                    None => write!(buf, "null")?,
                }
                if k != 3 {
                    write!(buf, ",")?;
                }
            }
            write!(buf, "]\n")?;
        }
        Ok(())
    }

    pub fn import<R: Read>(&'m self, buf: R) -> Result<(), Error> {
        let mut rel = Vec::new();
        let buf = io::BufReader::new(buf);
        for line in buf.lines() {
            let line = line.map_err(|e| Error::IO(e))?;
            let val: [(usize, u8); 4] = serde_json::from_str(&line).map_err(|e| Error::Serde(e))?;
            let q = self.make_quad();
            rel.push((q, val));
        }
        for &(q, rn) in rel.iter() {
            let [(i0, r0), (i1, r1), (i2, r2), (i3, r3)] = rn;
            let n0 = Node(rel[i0].0, r0);
            let n1 = Node(rel[i1].0, r1);
            let n2 = Node(rel[i2].0, r2);
            let n3 = Node(rel[i3].0, r3);
            q.set_all(&[n0, n1, n2, n3]);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    Serde(serde_json::Error),
    IO(io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use stringreader::StringReader;

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
    fn check_quadedge_ind() {
        let q: QuadEdge<(), ()> = QuadEdge::default();
        assert_eq!(q.ind(0), Node(&q, 0));
        assert_eq!(q.ind(1), Node(&q, 1));
        assert_eq!(q.ind(2), Node(&q, 2));
        assert_eq!(q.ind(3), Node(&q, 3));
        assert_eq!(q.ind(4), Node(&q, 0));
    }

    #[test]
    fn check_node_next() {
        let q: QuadEdge<(), ()> = QuadEdge::default();
        q.set_all(&[Node(&q, 3), Node(&q, 2), Node(&q, 1), Node(&q, 0)]);
        assert_eq!(q.ind(0).next(), Node(&q, 3));
        assert_eq!(q.ind(1).next(), Node(&q, 2));
        assert_eq!(q.ind(2).next(), Node(&q, 1));
        assert_eq!(q.ind(3).next(), Node(&q, 0));
        assert_eq!(q.ind(4).next(), Node(&q, 3));
    }

    #[test]
    fn check_node_rot() {
        let q: QuadEdge<(), ()> = QuadEdge::default();
        assert_eq!(q.ind(0).rot(), Node(&q, 1));
        assert_eq!(q.ind(1).rot(), Node(&q, 2));
        assert_eq!(q.ind(2).rot(), Node(&q, 3));
        assert_eq!(q.ind(3).rot(), Node(&q, 0));
        assert_eq!(q.ind(4).rot(), Node(&q, 1));
    }

    #[test]
    fn check_node_set() {
        let q: QuadEdge<(), ()> = QuadEdge::default();
        q.ind(0).set(Node(&q, 3));
        q.ind(1).set(Node(&q, 2));
        q.ind(2).set(Node(&q, 1));
        q.ind(3).set(Node(&q, 0));
        assert_eq!(q.ind(0).next(), Node(&q, 3));
        assert_eq!(q.ind(1).next(), Node(&q, 2));
        assert_eq!(q.ind(2).next(), Node(&q, 1));
        assert_eq!(q.ind(3).next(), Node(&q, 0));
        q.ind(4).set(Node(&q, 0));
        assert_eq!(q.ind(0).next(), Node(&q, 0));
    }

    #[test]
    fn check_node_swap() {
        let q: QuadEdge<(), ()> = QuadEdge::default();
        q.set_all(&[Node(&q, 0), Node(&q, 3), Node(&q, 2), Node(&q, 1)]);
        assert_eq!(q.ind(0).next(), Node(&q, 0));
        assert_eq!(q.ind(1).next(), Node(&q, 3));
        assert_eq!(q.ind(2).next(), Node(&q, 2));
        assert_eq!(q.ind(3).next(), Node(&q, 1));
        q.ind(0).swap(q.ind(2));
        q.ind(1).swap(q.ind(3));
        assert_eq!(q.ind(0).next(), Node(&q, 2));
        assert_eq!(q.ind(1).next(), Node(&q, 1));
        assert_eq!(q.ind(2).next(), Node(&q, 0));
        assert_eq!(q.ind(3).next(), Node(&q, 3));
    }

    #[test]
    fn check_node_splice_self_orig_dest() {
        let m: Manifold<(), ()> = Manifold::default();
        let q = m.make_quad();
        q.ind(0).splice(q.ind(2));
        assert_eq!(q.ind(0).next(), Node(&q, 2), "q.ind(0).next()");
        assert_eq!(q.ind(1).next(), Node(&q, 1), "q.ind(1).next()");
        assert_eq!(q.ind(2).next(), Node(&q, 0), "q.ind(2).next()");
        assert_eq!(q.ind(3).next(), Node(&q, 3), "q.ind(3).next()");
    }

    #[test]
    fn check_node_splice_cummulative_property() {
        let m: Manifold<(), ()> = Manifold::default();
        let q0 = m.make_quad();
        q0.ind(0).splice(q0.ind(2));
        assert_eq!(q0.ind(0).next(), Node(&q0, 2), "q0.ind(0).next()");
        assert_eq!(q0.ind(1).next(), Node(&q0, 1), "q0.ind(1).next()");
        assert_eq!(q0.ind(2).next(), Node(&q0, 0), "q0.ind(2).next()");
        assert_eq!(q0.ind(3).next(), Node(&q0, 3), "q0.ind(3).next()");

        let q1 = m.make_quad();
        q1.ind(2).splice(q1.ind(0));
        assert_eq!(q1.ind(0).next(), Node(&q1, 2), "q1.ind(0).next()");
        assert_eq!(q1.ind(1).next(), Node(&q1, 1), "q1.ind(1).next()");
        assert_eq!(q1.ind(2).next(), Node(&q1, 0), "q1.ind(2).next()");
        assert_eq!(q1.ind(3).next(), Node(&q1, 3), "q1.ind(3).next()");
    }

    #[test]
    fn check_node_splice_inverse_property() {
        let m: Manifold<(), ()> = Manifold::default();
        let q0 = m.make_quad();
        let q1 = m.make_quad();

        // Join nodes
        q0.ind(0).splice(q1.ind(0));
        assert_eq!(q0.ind(0).next(), Node(&q1, 0), "q0.ind(0).next()");
        assert_eq!(q0.ind(1).next(), Node(&q1, 3), "q0.ind(1).next()");
        assert_eq!(q0.ind(2).next(), Node(&q0, 2), "q0.ind(2).next()");
        assert_eq!(q0.ind(3).next(), Node(&q0, 1), "q0.ind(3).next()");
        assert_eq!(q1.ind(0).next(), Node(&q0, 0), "q1.ind(0).next()");
        assert_eq!(q1.ind(1).next(), Node(&q0, 3), "q1.ind(1).next()");
        assert_eq!(q1.ind(2).next(), Node(&q1, 2), "q1.ind(2).next()");
        assert_eq!(q1.ind(3).next(), Node(&q1, 1), "q1.ind(3).next()");
        // Split nodes
        q1.ind(0).splice(q0.ind(0));
        assert_eq!(q0.ind(0).next(), Node(&q0, 0), "q0.ind(0).next()");
        assert_eq!(q0.ind(1).next(), Node(&q0, 3), "q0.ind(1).next()");
        assert_eq!(q0.ind(2).next(), Node(&q0, 2), "q0.ind(2).next()");
        assert_eq!(q0.ind(3).next(), Node(&q0, 1), "q0.ind(3).next()");
        assert_eq!(q1.ind(0).next(), Node(&q1, 0), "q1.ind(0).next()");
        assert_eq!(q1.ind(1).next(), Node(&q1, 3), "q1.ind(1).next()");
        assert_eq!(q1.ind(2).next(), Node(&q1, 2), "q1.ind(2).next()");
        assert_eq!(q1.ind(3).next(), Node(&q1, 1), "q1.ind(3).next()");
    }

    #[test]
    fn check_manifold_make_quad() {
        let m: Manifold<(), ()> = Manifold::default();
        let q = m.make_quad();
        assert_eq!(q.ind(0).next(), Node(q, 0));
        assert_eq!(q.ind(1).next(), Node(q, 3));
        assert_eq!(q.ind(2).next(), Node(q, 2));
        assert_eq!(q.ind(3).next(), Node(q, 1));
    }

    #[test]
    fn check_manifold_export() {
        let m: Manifold<(), ()> = Manifold::default();
        let q0 = m.make_quad();
        let q1 = m.make_quad();
        q0.set_all(&[Node(q1, 3), Node(q0, 2), Node(q1, 1), Node(q0, 0)]);
        q1.set_all(&[Node(q0, 0), Node(q1, 1), Node(q0, 2), Node(q1, 3)]);
        let mut buf = io::BufWriter::new(Vec::new());
        m.export(&mut buf).expect("Export to buffer failed!");
        let bytes = buf.into_inner().expect("Conversion failed!");
        let result = String::from_utf8(bytes).expect("Invalid utf8!");
        let expected = "[[1,3],[0,2],[1,1],[0,0]]\n[[0,0],[1,1],[0,2],[1,3]]\n";
        assert_eq!(result, expected.to_string());
    }

    #[test]
    fn check_manifold_import() {
        let m: Manifold<(), ()> = Manifold::default();
        let s = "[[1, 3],[0,2],[1,1 ],[0,0]]\n[[ 0,0],[1,1],[0,2],[1,3] ]";
        let strrdr = StringReader::new(s);
        m.import(strrdr).expect("Import from buffer failed!");
        assert_eq!(m.quads.len(), 2);
        let q0 = m.qrefs.borrow()[0];
        let q1 = m.qrefs.borrow()[1];
        assert_eq!(q0.ind(0).next(), Node(q1, 3));
        assert_eq!(q0.ind(1).next(), Node(q0, 2));
        assert_eq!(q0.ind(2).next(), Node(q1, 1));
        assert_eq!(q0.ind(3).next(), Node(q0, 0));
        assert_eq!(q1.ind(0).next(), Node(q0, 0));
        assert_eq!(q1.ind(1).next(), Node(q1, 1));
        assert_eq!(q1.ind(2).next(), Node(q0, 2));
        assert_eq!(q1.ind(3).next(), Node(q1, 3));
    }
}

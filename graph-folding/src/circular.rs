use std::ops::Index;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Node(pub usize);

pub struct Data<T> {
  pub data: T,
  pub prev: Node,
  pub next: Node,
}

/// A very quick & dirty circular linked list.
/// Leaks any removed nodes until the list is dropped.
pub struct Circular<T> {
  v: Vec<Data<T>>,
}

impl<T> Circular<T> {
  pub fn new() -> Self {
    Self { v: vec![] }
  }

  fn m(&mut self, n: Node) -> &mut Data<T> {
    &mut self.v[n.0]
  }

  pub fn add_node(&mut self, data: T) -> Node {
    let n = self.v.len();
    self.v.push(Data {
      data,
      prev: Node(n),
      next: Node(n),
    });
    Node(n)
  }

  pub fn mut_data(&mut self, n: Node) -> &mut T {
    &mut self.v[n.0].data
  }

  /// Interchanges pointers to make a.next = b.
  /// Returns the old b.prev
  pub fn splice(&mut self, a: Node, b: Node) -> Node {
    let an = self[a].next;
    let bp = self[b].prev;
    self.m(a).next = b;
    self.m(b).prev = a;
    self.m(an).prev = bp;
    self.m(bp).next = an;
    bp
  }

  /// Split out the section of list between b and a,
  /// making b.next = a.
  /// Returns the old a.prev
  pub fn split(&mut self, a: Node, b: Node) -> Node {
    self.splice(b, a)
  }

  pub fn iter(&self, start: Node) -> impl Iterator<Item = Node> + '_ {
    let mut n = start;
    std::iter::once(start).chain(std::iter::from_fn(move || {
      n = self[n].next;
      if n == start {
        None
      } else {
        Some(n)
      }
    }))
  }
}

impl<T> Index<Node> for Circular<T> {
  type Output = Data<T>;

  #[inline(always)]
  fn index(&self, n: Node) -> &Self::Output {
    &self.v[n.0]
  }
}

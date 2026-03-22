use std::boxed;
#[allow(unused)]
use std::mem;

#[derive(Debug)]
pub struct List {
    head: Link,
}
type Link = Option<Box<Node>>;

#[derive(Debug)]
struct Node {
    elem: i32,
    next: Link,
}

impl List {
    fn new() -> Self {
        List { head: None }
    }

    fn push(&mut self, elem: i32) {
        eprintln!("before: {:?}", self);
        let new_node = Box::new(Node {
            elem: elem,
            next: self.head.take(),
        });
        eprintln!("after : self {:?} node: {:?}", self, new_node);

        self.head = Some(new_node)
    }

    fn pop(&mut self) -> Option<i32> {
        self.head.take().map(|node| {
            self.head = node.next;
            node.elem
        })
    }
}

impl Drop for List {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();

        while let Some(mut boxed_node) = cur_link {
            cur_link = mem::replace(&mut boxed_node.next, None);
        }
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        assert_eq!(list.pop(), None);

        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));

        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
    }
}

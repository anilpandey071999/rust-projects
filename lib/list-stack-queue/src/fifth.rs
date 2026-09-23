use std::cell;
use std::ptr::null_mut;
pub struct List<T> {
    head: Link<T>,
    tail: *mut Node<T>,
}

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: null_mut(),
        }
    }

    pub fn push(&mut self, elem: T) {
        let mut new_node = Box::new(Node { elem, next: None });

        let raw_tail: *mut _ = &mut *new_node;

        if !self.tail.is_null() {
            unsafe {
                (*self.tail).next = Some(new_node);
            }
        } else {
            self.head = Some(new_node);
        }

        self.tail = raw_tail;
    }

    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|head| {
            let head = *head;
            self.head = head.next;
            if self.head.is_none() {
                self.tail = null_mut();
            }
            head.elem
        })
    }
}

#[cfg(test)]
mod test {
    use std::{
        cell::{Cell, RefCell},
        collections::HashMap,
        rc::Rc,
        sync::{
            Arc, Mutex,
            atomic::{AtomicI32, Ordering::Relaxed},
            mpsc,
        },
        thread::spawn,
    };

    use super::List;
    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);

        // Check the exhaustion case fixed the pointer right
        list.push(6);
        list.push(7);

        // Check normal removal
        assert_eq!(list.pop(), Some(6));
        assert_eq!(list.pop(), Some(7));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn test_push_pop() {
        assert_eq!((3.14 + 1e20) - 1e20, 3.14);
    }

    #[test]
    fn test_cell() {
        let hashcell = Rc::new(RefCell::new(HashMap::new()));
        {
            let mut map = hashcell.borrow_mut();
            map.insert("key1", 10);
            map.insert("key2", 10);
            map.insert("key3", 10);
        }
        let total: i32 = hashcell.borrow().values().sum();
        println!("total: {total}");
        let a = Cell::new("g".to_uppercase().to_string());
        // let b = a.get();
    }
    #[test]
    fn cunter_rust_arc() {
        let count: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
        let mut handles = vec![];
        for _ in 0..3 {
            let count_clone = count.clone();
            let h = spawn(move || {
                let mut a = count_clone.lock().expect("");
                *a += 1;
            });
            handles.push(h);
        }
        for h in handles {
            h.join().expect("Thread panicked");
        }

        // 7. Verify the final value is 3
        assert_eq!(*count.lock().unwrap(), 3);
    }

    fn increatment(tx: mpsc::Sender<i32>, range: i32) {
        let mut counter = 0;
        for _ in 0..range {
            counter += 1;
        }
        tx.send(counter).unwrap();
    }
    #[test]
    fn cunter_rust_channel() {
        let target = 10_000;
        let (sender, rev) = mpsc::channel();
        let mut handles = vec![];
        for _ in 0..4 {
            let sender_clone = sender.clone();
            let h = spawn(move || {
                increatment(sender_clone, target / 4);
            });
            handles.push(h);
        }
        drop(sender);

        for h in handles {
            h.join().unwrap();
        }
        let sums: i32 = rev.iter().sum();
        println!("{sums}: {target}")
    }

    #[test]
    fn counter_atomic() {
        let target = 10_000;
        let counter = Arc::new(AtomicI32::new(0));
        let mut handler = vec![];
        for _ in 0..4 {
            let cunter_clone = counter.clone();
            let h = spawn(move || {
                for _ in 0..target / 4 {
                    cunter_clone.fetch_add(1, Relaxed);
                }
            });
            handler.push(h);
        }
        for h in handler {
            h.join().unwrap()
        }
        println!("{}: {target}", counter.load(Relaxed))
    }
}

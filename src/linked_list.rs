use std::{
    fmt::{Debug, Display, Write}, fs::File, io::Read, ptr::NonNull
};

pub struct LinkedList<T: Default> {
    head: NonNull<Node<T>>,
    tail: NonNull<Node<T>>,
    current: NonNull<Node<T>>,
    forward_iter: NonNull<Node<T>>,
    backward_iter: NonNull<Node<T>>,
    itered: bool,
}

struct Node<T> {
    next: NonNull<Node<T>>,
    prev: NonNull<Node<T>>,
    data: T,
}

unsafe impl<T: Send + Default> Send for LinkedList<T> {}
unsafe impl<T: Sync + Default> Sync for LinkedList<T> {}

impl<T> Node<T> {
    fn new(data: T) -> Self {
        Self {
            next: NonNull::dangling(),
            prev: NonNull::dangling(),
            data,
        }
    }

    fn set_next(&mut self, node: NonNull<Node<T>>) {
        self.next = node
    }

    fn set_prev(&mut self, node: NonNull<Node<T>>) {
        self.prev = node
    }

    fn get_next(&self) -> NonNull<Node<T>> {
        self.next
    }

    fn get_prev(&self) -> NonNull<Node<T>> {
        self.prev
    }
}

impl<T: Debug + Display> Debug for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.data, f)
    }
}

impl<T: Default> LinkedList<T> {
    pub fn new() -> Self {
        let head = Box::new(Node {
            next: NonNull::dangling(),
            prev: NonNull::dangling(),
            data: T::default(),
        });
        let tail = Box::new(Node {
            next: NonNull::dangling(),
            prev: NonNull::dangling(),
            data: T::default(),
        });
        let tail = NonNull::new(Box::into_raw(tail)).unwrap();
        let head = NonNull::new(Box::into_raw(head)).unwrap();
        unsafe {
            head.as_ptr().as_mut().unwrap().set_next(tail);
            tail.as_ptr().as_mut().unwrap().set_prev(head);
        }
        return LinkedList {
            current: head,
            head,
            tail,
            forward_iter: head,
            backward_iter: tail,
            itered: false,
        };
    }

    pub fn insert_before(&mut self, mut target: NonNull<Node<T>>, data: T) {
        assert!(target != self.head);
        let mut node = Node::new(data);
        node.set_next(target);
        unsafe {
            node.set_prev(target.as_ref().get_prev());
        }
        let new_box = Box::new(node);
        let new_node = NonNull::new(Box::into_raw(new_box)).unwrap();
        unsafe {
            target
                .as_ptr()
                .as_ref()
                .unwrap()
                .get_prev()
                .as_mut()
                .set_next(new_node);
            target.as_mut().set_prev(new_node)
        }
        self.current = new_node;
    }

    pub fn insert_after(&mut self, target: NonNull<Node<T>>, data: T) {
        assert!(target != self.tail);
        let mut node = Node::new(data);
        unsafe {
            node.set_next(target.as_ref().get_next());
            node.set_prev(target);
        }
        let new_box = Box::new(node);
        let new_node = NonNull::new(Box::into_raw(new_box)).unwrap();
        unsafe {
            target
                .as_ptr()
                .as_ref()
                .unwrap()
                .get_next()
                .as_mut()
                .set_prev(new_node);
            target.as_ptr().as_mut().unwrap().set_next(new_node);
        }
        self.current = new_node;
    }

    pub fn forward(&mut self) {
        if self.current != self.tail {
            unsafe { self.current = self.current.as_ref().get_next() }
        }
    }

    pub fn backward(&mut self) {
        if self.current != self.head {
            unsafe { self.current = self.current.as_ref().get_prev() }
        }
    }

    pub fn begin(&mut self) {
        self.current = self.head
    }

    pub fn end(&mut self) {
        self.current = self.tail
    }

    pub fn prepend(&mut self, data: T) {
        let was_empty = self.is_empty();
        self.insert_after(self.head, data);
        if was_empty {
            self.backward_iter = self.current;
        }
        self.forward_iter = self.current;
    }

    pub fn insert(&mut self, data: T) {
        self.insert_after(self.current, data);
    }

    pub fn append(&mut self, data: T) {
        let was_empty = self.is_empty();
        self.insert_before(self.tail, data);
        if was_empty {
            self.forward_iter = self.current;
        }
        self.backward_iter = self.current;
    }

    pub fn edit(&mut self, data: T) {
        if self.current != self.tail && self.current != self.head {
            self.remove();
            self.insert(data);
        }
    }

    fn init_iter_markers(&mut self) {
        unsafe {
            //if self.backward_iter.as_ref().get_next() == self.backward_iter {
            //    self.itered = true;
            //}
            self.backward_iter = self.backward_iter.as_ref().get_prev();
            self.forward_iter = self.forward_iter.as_ref().get_next();
        }
    }

    fn is_empty(&self) -> bool {
        unsafe { self.head.as_ref().get_next() == self.tail }
    }

    pub fn remove(&mut self) -> Option<Node<T>> {
        self.remove_node(self.current)
    }

    pub fn get(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            unsafe{
                Some(&self.current.as_ref().data)
            }
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            unsafe{
                Some(&mut(self.current.as_mut().data))
            }
        }
    }

    fn remove_node(&mut self, node: NonNull<Node<T>>) -> Option<Node<T>> {
        if node != self.tail && node != self.head {
            let old_node_ptr = node.as_ptr();
            let old_data = unsafe { old_node_ptr.read() };
            self.current = old_data.get_prev();
            unsafe {
                old_data.get_prev().as_mut().set_next(old_data.get_next());
                old_data.get_next().as_mut().set_prev(old_data.get_prev())
            }
            unsafe { drop(Box::from_raw(old_node_ptr)) };
            Some(old_data)
        } else {
            None
        }
    }
}

impl<T: Default> Iterator for LinkedList<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.itered || self.is_empty() {
            None
        } else {
            if self.backward_iter == self.forward_iter {
                self.itered = true;
            }
            let n = self.remove_node(self.forward_iter).unwrap();
            self.forward_iter = n.get_next();
            Some(n.data)
        }
    }
}

impl<T: Default> DoubleEndedIterator for LinkedList<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.itered || self.is_empty() {
            None
        } else {
            if self.backward_iter == self.forward_iter {
                self.itered = true;
            }
            let n = self.remove_node(self.backward_iter).unwrap();
            self.backward_iter = n.get_prev();
            Some(n.data)
        }
    }
}

impl<T: Default> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let head = self.head.as_ptr();
        let tail = self.tail.as_ptr();
        for _ in self {}
        unsafe {
            drop(Box::from_raw(head));
            drop(Box::from_raw(tail));
        }
    }
}

impl<T: Default + Debug + Display> Debug for LinkedList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('[').unwrap();
        let mut n = self.head;
        loop {
            n = unsafe { n.as_ref().get_next() };
            if n == self.tail {
                break;
            }
            unsafe {
                Debug::fmt( &n.as_ref().data, f).unwrap()
                //fmt(f).unwrap()
            }
            f.write_char(',').unwrap();
        }
        f.write_char(']')
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn headeqcurr() {
        let list = LinkedList::<char>::new();
        assert_eq!(list.head, list.current)
    }

    #[test]
    fn append() {
        let mut list = LinkedList::<char>::new();
        list.append('c');
        unsafe { assert_eq!(list.tail.as_ref().get_prev().as_ref().data, 'c') }
    }

    #[test]
    fn prepend() {
        let mut list = LinkedList::<char>::new();
        list.prepend('c');
        unsafe { assert_eq!(list.head.as_ref().get_next().as_ref().data, 'c') }
        list.prepend('d');
        unsafe {
            assert_eq!(list.head.as_ref().get_next().as_ref().data, 'd');
            assert_eq!(
                list.head
                    .as_ref()
                    .get_next()
                    .as_ref()
                    .get_next()
                    .as_ref()
                    .data,
                'c'
            )
        }
    }

    #[test]
    fn iterate_forward() {
        let mut list = LinkedList::<char>::new();
        list.append('b');
        list.append('c');
        list.append('d');
        list.append('e');
        list.prepend('a');
        list.append('f');

        assert_eq!(list.next(), Some('a'));
        assert_eq!(list.next(), Some('b'));
        assert_eq!(list.next(), Some('c'));
        assert_eq!(list.next(), Some('d'));
        assert_eq!(list.next(), Some('e'));
        assert_eq!(list.next(), Some('f'));
        assert_eq!(list.next(), None);
    }

    #[test]
    fn iterate_backward() {
        let mut list = LinkedList::<char>::new();
        list.append('b');
        list.append('c');
        list.append('d');
        list.append('e');
        list.prepend('a');
        list.append('f');

        assert_eq!(list.next_back(), Some('f'));
        assert_eq!(list.next_back(), Some('e'));
        assert_eq!(list.next_back(), Some('d'));
        assert_eq!(list.next_back(), Some('c'));
        assert_eq!(list.next_back(), Some('b'));
        assert_eq!(list.next_back(), Some('a'));
        assert_eq!(list.next_back(), None);
    }

    #[test]
    fn iteration() {
        let mut list = LinkedList::<char>::new();
        list.append('b');
        list.append('c');
        list.append('d');
        list.append('e');
        list.prepend('a');
        list.append('f');

        assert_eq!(list.next_back(), Some('f'));
        assert_eq!(list.next(), Some('a'));
        assert_eq!(list.next_back(), Some('e'));
        assert_eq!(list.next(), Some('b'));
        assert_eq!(list.next_back(), Some('d'));
        assert_eq!(list.next(), Some('c'));
        assert_eq!(list.next_back(), None);
        assert_eq!(list.next(), None);
    }

    #[test]
    fn edit() {
        let mut list = LinkedList::<char>::new();
        list.append('b');
        list.append('c');
        list.append('d');
        list.append('e');
        list.prepend('a');
        list.append('f');

        list.edit('g');
        assert_eq!(list.get(), Some(&'g'));

        list.backward();
        list.edit('p');
        assert_eq!(list.get(), Some(&'p'));

        list.forward();
        list.edit('z');
        assert_eq!(list.get(), Some(&'z'));


        println!("{:?}", list);
    }
}

mod linked_list;

use linked_list::LinkedList;



fn main() {
    let mut list = LinkedList::<char>::new();
    list.append('b');
    list.append('c');
    list.append('d');
    list.append('e');
    list.prepend('a');
    list.append('f');

    println!("{:?}", list)
}
use std::collections::HashMap;
use std::fmt::Debug;
use regex::bytes::Regex;

pub fn any() {
    use std::fmt::Debug;
    use std::any::Any;

    fn log<T: Any + Debug>(value: &T) {
        let value_any = value as &dyn Any;

        match value_any.downcast_ref::<String>() {
            Some(as_string) => {
                println!("String ({}): {}", as_string.len(), as_string);
            }
            None => {
                println!("{value:?}");
            }
        }
    }

    fn do_work<T: Any + Debug>(value: &T) {
        log(value);
    }

    fn main() {
        let my_string = "Hello World".to_string();
        do_work(&my_string);
        let my_i8: i8 = 100;
        do_work(&my_i8);
    }
}

#[test]
pub fn test_translate() {
    trait Showable {
        fn show(&self) {} // 要有self
    }

    impl dyn Showable {
        fn a() {}

        fn d(&self) {}
    }

    fn t<T>(a: &T) where T: Showable {
        a.show();
    }

    #[derive(Debug)]
    struct A {}
    impl Showable for A {}

    let a = A {};

    t(&a); // 引用也是要单独实现的

    fn print_it<T: Debug + 'static>(input: T) {
        println!("'static value passed in is: {:?}", input);
    }

    print_it(a);

    let mut d = 1;
    let d1 = &mut d;
    fn consume(d2: &mut i32) {
        *d2 = 2;
    }
    consume(d1); // 很奇怪这个不会被消化掉
    //let d4 = d1; // 如果d1 是 &mut 那么会转移到d4 因为&mut未实现copy
    println!("{}", *d1)

    // print_it1(a);
}

#[test]
fn string_int() {
    let a = u64::from_str_radix("-aa", 16).unwrap();
}

use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::h2_rust_common::Nullable;
use crate::h2_rust_common::Nullable::NotNull;

fn check<T: BorrowMut<[i32]>>(mut v: T) {
    assert_eq!(&mut [1, 2, 3], v.borrow_mut());
}

fn main() {
    let mut v = vec![1, 2, 3];

    assert_eq!(
        &mut [1, 2, 3],
        <Vec<i32> as BorrowMut<[i32]>>::borrow_mut(&mut v)
    );

    check(v);
}

#[test]
fn test_arc_mut() {
    struct User {
        salary: u8,
    }

    let a = Arc::new(RefCell::new(User { salary: 1 }));
    (&*a).borrow_mut().salary = 100;

    let b = Mutex::new(User { salary: 1 });
    b.lock().unwrap().salary = 100;
}

#[test]
fn test_multi_thread_refcell() {
    struct Company {
        level: i64,
    }

    struct User {
        salary: u64,
        company: Arc<RefCell<Nullable<Company>>>,
    }

    let a = Arc::new(Mutex::new(User { salary: 1, company: Arc::new(RefCell::new(NotNull(Company { level: 2 }))) }));
    let b = a.clone();

    thread::spawn(move || {
        println!("{}", b.lock().unwrap().salary);
    });
}
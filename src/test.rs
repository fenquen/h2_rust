use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
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
        // fn aa(){} 报错需要去改为 fn aa() where Self: Sized{}
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
}

#[test]
fn string_int() {
    let a = u64::from_str_radix("-aa", 16).unwrap();
}

use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use atomic_refcell::AtomicRefCell;
use crossbeam::atomic::AtomicCell;
use crate::h2_rust_common::{h2_rust_utils, Integer, Nullable};
use crate::h2_rust_common::Nullable::NotNull;
use crate::mvstore::data_utils;

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

#[derive(Default)]
struct Company {
    pub level: i64,
}

struct User {
    salary: u64,
    company: Arc<Nullable<Company>>,
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
    /*let a = Arc::new(Mutex::new(User { salary: 1, company: Arc::new(RefCell::new(NotNull(Company { level: 2 }))) }));
    let b = a.clone();

    thread::spawn(move || {
        println!("{}", b.lock().unwrap().salary);
    });*/

    let box_ = Box::new(Company { level: 1 });

    let a = &*box_;
    let d = Box::into_raw(box_);
    let dd = d as usize;
    unsafe { (*d).level = 1000; }

    thread::spawn(move || {
        let dd = dd as *mut Company;
        unsafe { (*dd).level = 10000 }
    }).join().unwrap();

    println!("{}", unsafe { (*d).level });
}

#[test]
fn test_crossbeam() {
    let a = Arc::new(AtomicCell::new(Company { level: 1 }));
    let aa = a.clone();

    thread::spawn(move || {
        let s = aa.as_ptr();
        unsafe { (*s).level = 100 }
    }).join().unwrap();

    println!("{}", unsafe { (*(a.as_ptr())).level })
}

#[test]
fn test_atomic_refcell() {
    fn change_a(this: Arc<AtomicRefCell<Nullable<Company>>>) {
        let mut binding = (&*this).borrow_mut();
        let company = binding.unwrap_mut();
    }

    let a = Arc::new(AtomicRefCell::new(NotNull(Company::default())));
    let aa = a.clone();

    let mut binding = (&*a).borrow_mut();
    let company = binding.unwrap_mut();
    company.level = 9;

    drop(binding);
    change_a(aa);
    let mut binding = (&*a).borrow_mut();
    let company = binding.unwrap_mut();
    company.level = 19;
}

#[test]
fn test_generic() {
    struct Company<T> {
        member: T,
    }

    impl<T> Company<T> {
        fn a(&self) {}
    }
}

#[test]
fn test_hash_map_any() {
    let mut map = HashMap::<String, Box<dyn Any>>::new();
    map.insert("value".to_string(), Box::new("171".to_string()));

    let a = data_utils::get_config_int_param(&map, "value", 1);
    println!("{}", a);

    match h2_rust_utils::get_from_map::<String>(&map, "value") {
        Nullable::NotNull(s) => { println!("not null :{}", s) }
        Nullable::Null => { println!("null") }
    }
}

fn test_abstract() {
    trait Locale {
        fn format_date() -> String;
    }

    struct Greeting<LOCALE: Locale> {
        name: String,
        locale: PhantomData<LOCALE>, // needed to satisfy compiler
    }

    impl<LOCALE: Locale> Greeting<LOCALE> {
        pub fn new(name: String) -> Self {
            Self {
                name,
                locale: PhantomData,
            }
        }

        pub fn greet(&self) {
            format!("Hello {}\nToday is {}", self.name, LOCALE::format_date());
        }
    }

    pub struct UsaLocale;

    impl Locale for UsaLocale {
        fn format_date() -> String {
            // somehow get year, month, day
            format!("{}/{}", "month", "year")
        }
    }

    type UsaGreeting = Greeting<UsaLocale>;
}

#[test]
fn test_parameter_func() {
    struct A<T: ?Sized> {
        pub t: T,
    }

    struct B {
        pub a: A<Arc<dyn Any>>,
    }

    impl B {
        pub fn add(&mut self, a: A<Arc<dyn Any>>) {
            self.a = a;
        }

        pub fn read<T: 'static>(&self) -> &T {
            self.a.t.deref().downcast_ref::<T>().unwrap()
        }
    }

    let a: A<Arc<dyn Any>> = A { t: Arc::new(1) };
    let b = B { a };

    let d = b.read::<Integer>();
    println!("{}", d);

    let a = Nullable::<Integer>::Null;
    let a: Nullable<Integer> = a.clone();
}

#[test]
fn test_duplicate_name_abstract_func() {
    trait A {
        fn show(&self);
        fn show1(&self);
    }

    trait B: A {
        fn show(&self);
    }

    struct C;

    impl A for C {
        fn show(&self) {
            println!("{}", "A::show()")
        }

        fn show1(&self) {
            println!("{}", "A::show1()")
        }
    }

    impl B for C {
        fn show(&self) {
            println!("{}", "b")
        }
    }

    let c = C;
    let c = &c as &dyn A;
    c.show1();

    struct User{}
    impl User{
        pub fn show(mut self){}
    }

    let user = User{};

    user.show();


    let mut a = Option::Some(1);
    let a = a.as_mut().unwrap();
}
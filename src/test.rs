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
use std::cell::{Cell, RefCell, UnsafeCell};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, mpsc, Mutex, RwLock};
use std::sync::atomic::{AtomicPtr, AtomicU64};
use std::{alloc, mem, ptr, thread};
use std::alloc::Layout;
use std::rc::Rc;
use std::task::ready;
use std::time::Duration;
use atomic_refcell::AtomicRefCell;
use crossbeam::atomic::AtomicCell;
use parking_lot::RwLockWriteGuard;
use crate::h2_rust_cell_mut_call;
use crate::h2_rust_common::{h2_rust_utils, Integer, Nullable};
use crate::h2_rust_common::h2_rust_cell::H2RustCell;
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
        Some(s) => { println!("not null :{}", s) }
        None => { println!("null") }
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

    struct User {}
    impl User {
        pub fn show(self) {}
    }

    let user = User {};

    user.show();

    let mut a = Option::Some(1);
    let a = a.as_mut().unwrap();
}

#[test]
fn test_closure() {
    let mut a = 1;
    let mut f = || { a = a + 1; }; // 闭包里的其实是&mut
    f();
    println!("{}", a);

    let mut b = String::from("hello");

    // rust 自动检测到 pushed_data 这个匿名函数要修改其外部的环境变量.
    // 因此自动推理出 pushed_data 是一个 FnMut 匿名函数.
    let pushed_data = || {
        b.push_str(" world!");

        // 由于rust的 mutable 原则是, 只允许一个mut引用, 因此 变量 b 不能 再被其他代码引用, 所以这里要返回更改后
        b // 要是返回b的话 函数变量前边不用加上mut 不然要加上和上边的相同
    };
    pushed_data();
    // println!("{}",b);

    struct Buffer<'a> {
        buf: &'a [u8],
        pos: usize,
    }

    impl<'a> Buffer<'a> {
        fn new(b: &'a [u8]) -> Buffer {
            Buffer { buf: b, pos: 0 }
        }

        fn read_bytes(&mut self) -> &'a [u8] {
            self.pos += 3;
            &self.buf[self.pos - 3..self.pos]
        }
    }
}

#[test]
fn test_atomic_ptr() {
    struct A {
        value: Integer,
    }

    let mut a = A { value: 1 };
    let a = AtomicPtr::new(&mut a);
}

fn test_struct_combination() {
    /// https://stackoverflow.com/questions/32552593/is-it-possible-for-one-struct-to-extend-an-existing-struct-keeping-all-the-fiel
    ///
    /// The good part is that you don't have to go through the pain of forwarding methods in Dog to methods in Animal; you can use them directly inside impl Animal<Dog>.
    /// Also, you can access any fields defined in Animal from any method of Animal<Dog>.
    ///
    /// The bad part is that your inheritance chain is always visible (that is, you will probably never use Dog in your code, but rather Animal<Dog>).
    /// Also, if the inheritance chain is long, you might get some very silly, long-winded types, like Animal<Dog<Chihuahua>>. I guess at that point a type alias would be advisable.
    trait AnimalTrait {
        fn show(&self);// 抽象类中的abstract函数
    }

    struct Animal<T: AnimalTrait> {
        name: String,
        age: i64,
        actual: T,
    }

    // implement the 'breathe' method for all animals
    impl<T: AnimalTrait> Animal<T> {
        fn breathe(&self) { // 抽象类中的非abstract函数
            println!("{}", "breath");
            self.actual.show()
        }
    }

    impl Animal<Dog> {
        pub fn bark(&self) -> String { // 实现类中自个的函数
            return "bark!".to_owned();
        }
    }

    struct Dog {
        favorite_toy: String,
    }

    impl AnimalTrait for Dog {
        fn show(&self) { // 只能访问到自己的field 如何应对
            println!("{}", self.favorite_toy);
        }
    }
}

#[test]
fn test_abstract0() {
    trait FatherCommon {
        fn show(&self) {
            println!("{}", "father show");
        }

        fn change_self(&mut self);
    }

    struct Father {
        pub hometown: String,
    }

    impl FatherCommon for Father {
        fn change_self(&mut self) {
            self.hometown = "change".to_string();
        }
    }

    struct Son {
        pub father: Father,
        pub achievement: String,
    }

    impl Son {
        fn private_func(&self) {
            println!("{}", "son private");
        }
    }

    impl FatherCommon for Son {
        fn show(&self) {
            self.father.show();
            println!("{}", "show son");
        }

        fn change_self(&mut self) {
            self.achievement = "change".to_string();
        }
    }

    let son = Son {
        father: Father {
            hometown: "hometown".to_string(),
        },
        achievement: "achievement".to_string(),
    };

    fn operate(a: Arc<AtomicRefCell<dyn FatherCommon>>) {
        let mut a = (&*a).borrow_mut();
        //  let a = a.deref_mut();
        a.change_self();
        a.show();
    }

    operate(Arc::new(AtomicRefCell::new(son)));
}

#[test]
fn test_drop() {
    struct User {}

    impl Drop for User {
        fn drop(&mut self) {
            println!("{}", "user drop");
        }
    }

    struct Company {
        user: User,
    }

    impl Drop for Company {
        fn drop(&mut self) {
            println!("{}", "company drop");
        }
    }

    fn a() -> User {
        User {}
    }

    let user = a(); // 如是a()会直接调用 User的drop()
    println!("{}", "aaaaaaaaaa");

    let company = Company { user };
}

#[test]
fn test_hashmap_multi_thread() {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex, RwLock},
        thread,
        time::Duration,
    };

    fn main() {
        let data = Arc::new(RwLock::new(HashMap::new()));

        let threads: Vec<_> = (0..10)
            .map(|i| {
                let data = Arc::clone(&data);
                thread::spawn(move || worker_thread(i, data))
            })
            .collect();

        for t in threads {
            t.join().expect("Thread panicked");
        }

        println!("{:?}", data);
    }

    fn worker_thread(id: u8, data: Arc<RwLock<HashMap<u8, Mutex<i32>>>>) {
        loop {
            // Assume that the element already exists
            let read_lock = data.read().expect("RwLock poisoned");

            if let Some(element) = read_lock.get(&id) {
                let mut element = element.lock().expect("Mutex poisoned");

                // Perform our normal work updating a specific element.
                // The entire HashMap only has a read lock, which
                // means that other threads can access it.
                *element += 1;
                thread::sleep(Duration::from_secs(1));

                return;
            }

            // If we got this far, the element doesn't exist

            // Get rid of our read lock and switch to a write lock
            // You want to minimize the time we hold the writer lock
            drop(read_lock);

            let mut write_lock = data.write().expect("RwLock poisoned");

            // We use HashMap::entry to handle the case where another thread
            // inserted the same key while where were unlocked.
            thread::sleep(Duration::from_millis(50));
            write_lock.entry(id).or_insert_with(|| Mutex::new(0));
            // Let the loop start us over to try again
        }
    }
}

#[test]
fn test_arc_swap() {
    use std::sync::Arc;
    use arc_swap::ArcSwap;

    fn main() {
        let config = ArcSwap::from(Arc::new(String::default()));
        thread::scope(|scope| {
            scope.spawn(|| {
                let new_conf = Arc::new("New configuration".to_owned());
                config.store(new_conf);
            });

            for _ in 0..10 {
                scope.spawn(|| {
                    loop {
                        let cfg = config.load();
                        if !cfg.is_empty() {
                            assert_eq!(**cfg, "New configuration");
                            return;
                        }
                    }
                });
            }
        });
    }
}

#[test]
fn test_simple_conversion() {
    use usync::RwLock;
    let rw_lock = RwLock::new(1);
    let a = rw_lock.write();
    rw_lock.write();
    println!("{}", 16usize.trailing_zeros());
}

#[test]
fn test_overlapping() {
    pub trait Shower {
        fn show(&mut self);
    }

    struct Company {
        name: String,
        user: Option<Arc<H2RustCell<dyn Shower>>>,
    }

    impl Shower for Company {
        fn show(&mut self) {
            println!("{}", "company show");
        }
    }

    struct User {
        name: String,
        belonging_company: Option<Arc<H2RustCell<dyn Shower>>>,
    }

    impl Shower for User {
        fn show(&mut self) {
            if self.belonging_company.is_some() {
                h2_rust_cell_mut_call!(self.belonging_company,show);
            }
            println!("{}", "user show")
        }
    }

    let company = Company {
        name: "company".to_string(),
        user: Default::default(),
    };
    let company_wrapper_arc = Arc::new(H2RustCell::new(company));

    let user = User {
        name: "user".to_string(),
        belonging_company: Default::default(),
    };
    let user_wrapper_arc = Arc::new(H2RustCell::new(user));

    // 该字段被替换会调用其析构函数
    user_wrapper_arc.get_ref_mut().belonging_company = Some(company_wrapper_arc.clone());
    company_wrapper_arc.get_ref_mut().user = Some(user_wrapper_arc.clone());

    let clone = user_wrapper_arc.clone();
    let join_handle = thread::spawn(move || {
        (&*clone).get_ref_mut().show();
    });

    (&*user_wrapper_arc).get_ref_mut().show();
    join_handle.join();
}

#[test]
fn test_a() {
    struct User(String);

    let a = Arc::new(User("a".to_string()));

    let join_handle = thread::spawn(move || {
        let a = Arc::clone(&a);
        println!("{}", a.0);
    });

    join_handle.join();
}

#[test]
fn test_lock() {
    #[derive(Default)]
    struct Company {
        count: Integer,
        mutex: Mutex<()>,
    }

    impl Company {
        fn set(&mut self) {
            self.count = 1;
        }
    }

    let a = H2RustCell::new(Company::default());

    let a = a.get_ref_mut();

    let guard = a.mutex.lock().unwrap();
    a.count = 1;
    // a.set(); // 会有错误 不可变引用的时候不能有可变引用

    let c = Cell::new("asdf");
    let one = c.get();
    c.set("qwer");
    let two = c.get();
}

#[test]
fn test_cast() {
    #[derive(Default)]
    struct User {
        value: Integer,
    }

    trait ContainerTrait<T> {
        fn show(&self, t: T);
    }

    #[derive(Default)]
    struct Container<T> (T);

    /* impl ContainerTrait<Arc<dyn Any>> for Container<Arc<dyn Any>> {
         fn show(&self, t: Arc<dyn Any>) {
             todo!()
         }
     }*/

    impl<T> ContainerTrait<T> for Container<T> {
        fn show(&self, t: T) {
            todo!()
        }
    }

    impl dyn ContainerTrait<Arc<dyn Any>> {
        pub fn show1(&self) {
            println!("{}", "dyn ContainerTrait");
        }

        pub fn cast<T>(&self) {}
    }

    let a = Arc::new(User::default());//as Arc<dyn Any>;


    let object = Arc::new(H2RustCell::new(Container(a))) as Arc<H2RustCell<dyn ContainerTrait<Arc<User>>>>;

    fn cast<T>(object: Arc<H2RustCell<dyn ContainerTrait<Arc<dyn Any>>>>) {
        let a = object.get_ref();
    }

    //let container = Container(User::default());
    /// let container = &object as &dyn Any;

    // let a = &a as &dyn Any;
    // println!("{}", a.downcast_ref::<Arc<dyn Any>>().is_some());

    // Option<Arc<H2RustCell<dyn ContainerTrait<Arc<dyn Any>>>>>>

    enum Car {
        TOYOTA(Toyota),
        RENO(Reno),
    }

    impl Car {
        pub fn run(&self) {
            match self {
                Self::TOYOTA(a) => { println!("{}", "toyota") }
                Self::RENO(a) => { println!("{}", "reno") }
            }
        }
    }

    struct Toyota {}

    struct Reno {}

    let a = Car::TOYOTA(Toyota {});
    a.run();
}

#[test]
fn test_int_hex() {
    let a = format!("{:x}", 27);
    println!("{}", a);
}

#[test]
fn test_read_ptr() {
    struct Car(u8);
    let x = Car(9);
    let y = &x as *const Car;
    println!("{}", y as usize);

    let y0 = unsafe { ptr::read(y) };

    println!("{}", x.0);
    println!("{}", &y0 as *const Car as usize)
}

#[test]
fn test_ref_cell_replace() {
    struct A(Integer);
    impl Drop for A {
        fn drop(&mut self) {
            println!("{}", "drop A")
        }
    }

    let c = H2RustCell::new(A(1));
    println!("{}", c.get_addr());
    *c.get_ref_mut() = A(2); // 容器的内容整个替换了会使得老的drop
    (*c.get_ref_mut()).0 = 21;
    println!("{}", c.get_addr()); // 地址不变

    println!("{}", "00000000000000");
    println!("{}", c.get_ref().0);
}
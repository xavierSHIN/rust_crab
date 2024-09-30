// TODO:: 

use tokio;
use std::thread;
use crate::utils; // Adjust the path based on the actual location of utils
//use super::*;
use std::sync::{Arc, Mutex};
use std::rc::Rc;

/// 为什么要使用Rc?
/// Rust 所有权机制要求一个值只能有一个所有者, 而当我们要实现一个值多个所有者时，就需要使用Rc
/// 即：共享数据！
/// Rc<T> 是指向底层数据的不可变的引用
/// 当我们希望在堆上分配一个对象供程序的多个部分使用且无法确定哪个部分最后一个结束时
/// ，就可以使用 Rc 成为数据值的所有者
/// Rc::clone 不是所有的clone都是深拷贝。这里的clone仅仅复制了智能指针并增加了引用计数，并没有克隆底层数据
pub fn show_rc() {
        let a = Rc::new(String::from("test ref counting"));
        println!("count after creating a = {}", Rc::strong_count(&a));
        let b =  Rc::clone(&a);
        println!("count after creating b = {}", Rc::strong_count(&a));
        {
            let c =  Rc::clone(&a);
            println!("count after creating c = {}", Rc::strong_count(&c));
        }
        println!("count after c goes out of scope = {}", Rc::strong_count(&a));
}

#[tokio::main]
pub async fn show_concurrent() {
    println!("show_concurrent --basic version");
    let mut handles = Vec::new();
    for idx in 0..4 {
        let handle = tokio::spawn(async move {
            let s = utils::generate_random_utf8_string(10);
            println!("idx: {}, string: {}", idx, s);
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.await.unwrap();
    }
    //utils::generate_random_utf8_string(10);
} 


/// Rc是单线程版本的，而Arc则是在多线程中实现了共享'所有权'
/// Rc<T>无法在线程中传输，因为它没有实现Send特征
/// 
/// 
/// Mutex 实现了Mut
/// Mutex让多个线程并发的访问同一个值变成了排队访问：同一时间，只允许一个线程A访问该值
/// 
/// 
/// 由于子线程需要通过move拿走锁的所有权，因此我们需要使用多所有权来保证每个线程都拿到数据的独立所有权
/// m.lock()向m申请一个锁, 该方法会阻塞当前线程，直到获取到锁
/// ，因此当多个线程同时访问该数据时，只有一个线程能获取到锁

#[tokio::main]
pub async fn show_concurrent_nonarc() {
    println!("show_concurrent --nonArc version--------------------------------------------");
    let mut handles = Vec::new();
    //let example_vec = Arc::new( vec![1, 2, 3, 4, 5]);
    let example_vec = vec![1, 2, 3, 4, 5];
    for idx in 0..4 {
        //let example_vec_clone = Arc::clone(&example_vec);
        let example_vec_clone = example_vec.clone();
        // need clone before moved to closure
        let handle = thread::spawn(move|| {
            let s = utils::generate_random_utf8_string(10);
            //println!("idx: {}, string: {}", idx, s);
            println!("{idx}: example_vec: {:?}", example_vec_clone[idx]);
            println!("{idx}: example_vec_ptr: {:?}", example_vec_clone.as_ptr());
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    println!("example_vec: {:?}", example_vec);
}
#[tokio::main]
pub async fn show_concurrent_arc() {
    println!("show_concurrent --Arc version--------------------------------------------");
    let mut handles = Vec::new();
    let example_vec = Arc::new( vec![1, 2, 3, 4, 5]);
    //let example_vec = vec![1, 2, 3, 4, 5];
    for idx in 0..4 {
        let example_vec_clone = Arc::clone(&example_vec);
        //let example_vec_clone = example_vec.clone();
        // need clone before moved to closure
        let handle = thread::spawn(move|| {
            let s = utils::generate_random_utf8_string(10);
            //println!("idx: {}, string: {}", idx, s);
            println!("{idx}: example_vec: {:?}", example_vec_clone[idx]);
            println!("{idx}: example_vec_ptr: {:?}", example_vec_clone.as_ptr());
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    println!("example_vec: {:?}", example_vec);
}



#[tokio::main]
pub async fn show_concurrent_mod() {
    println!("show_concurrent --modify version----------------------------------------------");
    let mut handles = Vec::new();
    //let example_vec = Arc::new( vec![1, 2, 3, 4, 5]);
    let example_vec = Vec::new();
    for idx in 0..4 {
        //let example_vec_clone = Arc::clone(&example_vec);
        let mut example_vec_clone = example_vec.clone();
        // need clone before moved to closure
        let handle = thread::spawn(move|| {
            let s = utils::generate_random_utf8_string(10);
            //println!("idx: {}, string: {}", idx, s);
            example_vec_clone.push((idx, s));
            println!("{idx}: example_vec: {:?}", example_vec_clone  );
            println!("{idx}: example_vec_ptr: {:?}", example_vec_clone.as_ptr());

        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    println!("example_vec: {:?}", example_vec);
    //utils::generate_random_utf8_string(10);
}

#[tokio::main]
pub async fn show_concurrent_mutex() {
    println!("show_concurrent --mutex version----------------------------------------------");
    let mut handles = Vec::new();
    //let example_vec = Arc::new( vec![1, 2, 3, 4, 5]);
    let example_vec = Arc::new(Mutex::new(Vec::new())) ;
    for idx in 0..4 {
        //let example_vec_clone = Arc::clone(&example_vec);
        let mut example_vec_clone = example_vec.clone();
        //let mut example_vec_clone = Arc::clone(&example_vec);
        // need clone before moved to closure
        let handle = thread::spawn(move|| {
            let s = utils::generate_random_utf8_string(10);
            //println!("idx: {}, string: {}", idx, s);
            // 解锁
            let mut example_vec_clone = example_vec_clone.lock().unwrap();
            example_vec_clone.push((idx, s));
            println!("{idx}: example_vec: {:?}", example_vec_clone);
            println!("{idx}: example_vec_ptr: {:?}", example_vec_clone.as_ptr());

        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    println!("example_vec: {:?}", example_vec);
    //utils::generate_random_utf8_string(10);
}
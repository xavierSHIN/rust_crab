use anyhow::Result;
use std::{sync::mpsc, thread, time::Duration};
//use rand::Rng;
use indicatif::ProgressBar;
// todo:: add indi::processBar DONE
// todo:: different thread different color？
// todo:: map_err(|e| anyhow!("Thread join error: {:?}", e))? applying chacha20poly1305
// map_err 是转化error type from one to another
//const NUM_PRODUCERS: usize = 3;

/// Create multiple producer threads and a single consumer thread
#[allow(dead_code)]
#[derive(Debug)]
pub struct Msg {
    idx: usize,
    value: usize,
}
impl Msg {
    fn new(idx: usize, value: usize) -> Self {
        Self { idx, value }
    }
}
fn producer(idx: usize, tx: mpsc::Sender<Msg>) -> Result<()> {
    //let deps = 2u64.pow(32) - 1; // Adjusted for demonstration purposes
    //let pb = ProgressBar::new(deps);
    let mut count: u32 = 0;
    loop {
        let deps = 2u64.pow(16)/5 - 1; // Adjusted for demonstration purposes
        let pb = ProgressBar::new(deps);
        let value = rand::random::<u8>();
        count = count.wrapping_add(value as u32);
        pb.inc(count as u64);
        // todo:: 如何把上下两个一起打包？
        println!("producer '{}': progress increase by:{}, total is:{}",idx, value, count);

        tx.send(Msg::new(idx, value as usize))?;
        // sleep_time 范围限制的太小会立马结束
        let judge_var = rand::random::<u16>();
        thread::sleep(Duration::from_millis(10*value as u64));
        // Randomly exit the producer
        if judge_var % 10 == 0 {
            println!("producer '{}' exit", idx);
            pb.finish_with_message("done");
            // pb finish 就是把Bar灌满
            break;
        }
        //pb.finish_with_message("done_1");
    }
    //pb.finish_with_message("done_2");
    Ok(())
}

fn consumer(rx: mpsc::Receiver<Msg>) -> Result<()> {
    // 因为此处rx 对应多个tx_clone, 所以需要用loop
    for received in rx {
        println!("rx Received: {:?}", received);
    }
    println!("consumer exit");
    Ok(())
}

pub fn main_thread(num_thread:usize) -> Result<()> {
    // Create a channel
    let (tx, rx) = mpsc::channel();
    // mspc：：channel() 用来实现 不同线程之间的通信
    // Spawn producer threads
    let mut handles = vec![];
    for i in 0..num_thread {
        let tx_clone = tx.clone();
        // 因为 move闭包捕获了tx所有权,所以需要clone
        let handle = thread::spawn(move || {
            //producer(i, tx_clone) //.map_err(|e| anyhow!("Thread join error: {:?}", e));
            //pb.finish_with_message("done_done");
            if let Err(e) = producer(i, tx_clone) {
                eprintln!("Producer thread error: {:?}", e);
            }
        }
        );
        handles.push(handle);
    }


    // Spawn consumer thread
    let consumer_handle = thread::spawn(move || {
        //consumer(rx)//.map_err(|e| anyhow!("Thread join error: {:?}", e));
        if let Err(e) = consumer(rx) {
            eprintln!("Producer thread error: {:?}", e);
        }
    });

    // Wait for all producer threads to finish
    for handle in handles {
        if let Err(e) = handle.join() {
            eprintln!("Thread join error: {:?}", e);
        }
        //handle.join()//.map_err(|e| anyhow!("Thread join error: {:?}", e))?;
    }
    // Drop the original sender to close the channel
    drop(tx);

    // Wait for the consumer thread to finish
    let secret = if let Err(e)  = consumer_handle.join() {
        eprintln!("Thread join error: {:?}", e);
    };//.map_err(|e| anyhow!("Thread join error: {:?}", e))?;
    
    println!("secret: {:#?}", secret);

    Ok(())


}


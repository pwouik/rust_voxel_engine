use std::thread::JoinHandle;
use std::sync::{mpsc, Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use crossbeam_channel;

pub struct ThreadPool<T:Send+'static,U:Send+'static>{
    sender:crossbeam_channel::Sender<T>,
    sender_thread:mpsc::Sender<U>,
    thread_handles:Vec<JoinHandle<()>>,
    running:Arc<AtomicBool>
}
impl<T:Send+'static,U:Send+'static> ThreadPool<T,U>{
    pub fn new(function:fn(T)->U)->(mpsc::Receiver<U>,Self){
        let (sender,receiver_thread) = crossbeam_channel::unbounded() as (crossbeam_channel::Sender<T>,crossbeam_channel::Receiver<T>);
        let (sender_thread,receiver) = mpsc::channel() as (mpsc::Sender<U>,mpsc::Receiver<U>);
        let mut thread_handles=Vec::new();
        let running = Arc::new(AtomicBool::new(true));
        for _ in 0..4{
            let loc_receiver = receiver_thread.clone();
            let loc_sender= sender_thread.clone();
            let loc_running= running.clone();
            thread_handles.push(thread::spawn(move || {
                while loc_running.load(Ordering::Relaxed) {
                    let input = loc_receiver.try_recv();
                    match input{
                        Ok(input)=>{
                            loc_sender.send(function(input)).unwrap();
                        }
                        Err(_)=>{thread::sleep(Duration::from_millis(20));}
                    }

                }
            }));
        }
        (receiver,
        ThreadPool{
            sender,
            sender_thread,
            thread_handles,
            running
        })
    }
    pub fn send(&mut self, input:T) -> Result<(), crossbeam_channel::TrySendError<T>> {
        self.sender.try_send(input)
    }
    pub fn pass(&mut self,output:U){
        self.sender_thread.send(output).unwrap();
    }
}
impl<T:Send+'static,U:Send+'static> Drop for ThreadPool<U,T> {
    fn drop(&mut self) {
        self.running.store(false,Ordering::Relaxed);
        for _i in 0..4{
            self.thread_handles.remove(0).join().unwrap();
        }
        println!("threadpool joined");
    }
}
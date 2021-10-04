use std::thread::JoinHandle;
use std::sync::{mpsc, Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use std::sync::mpsc::TryRecvError;
use crossbeam::channel;

pub struct ThreadPool<T:Send+'static,U:Send+'static>{
    sender:crossbeam::channel::Sender<T>,
    receiver:mpsc::Receiver<U>,
    thread_handles:Vec<JoinHandle<()>>,
    running:Arc<AtomicBool>
}
impl<T:Send+'static,U:Send+'static> ThreadPool<T,U>{
    pub fn new(function:fn(T)->U)->Self{
        let (sender,receiver_thread) = channel::bounded(30) as (channel::Sender<T>,channel::Receiver<T>);
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
        ThreadPool{
            sender,
            receiver,
            thread_handles,
            running
        }
    }
    pub fn send(&mut self, input:T) -> Result<(), channel::TrySendError<T>> {
        self.sender.try_send(input)
    }
    pub fn get(&mut self) -> Result<U, TryRecvError> {
        self.receiver.try_recv()
    }
}
impl<T:Send+'static,U:Send+'static> Drop for ThreadPool<U,T> {
    fn drop(&mut self) {
        self.running.store(false,Ordering::Relaxed);
        for i in 0..4{
            self.thread_handles.remove(0).join().unwrap();
        }
    }
}
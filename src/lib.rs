pub mod pool {
    use std::sync::mpsc::{self, Receiver, Sender};
    use std::sync::{Arc, Mutex};
    use std::thread::{self, JoinHandle};
    type Job = Box<dyn FnOnce() + 'static + Send>;
    type SafeReceiver = Arc<Mutex<Receiver<Job>>>;

    struct Worker {
        id: usize,
        thread: Option<JoinHandle<()>>,
    }

    impl Worker {
        fn new(id: usize, receiver: SafeReceiver) -> Self {
            // thread in Thread Pool
            let thread = thread::spawn(move || loop {
                // first, the channle blocks thread
                // the thead run, even when channle receive job,
                let job = receiver.lock().unwrap().recv().unwrap();
                println!("worker {} got a job;", id);
                job();
            });
            Worker {
                id,
                thread: Some(thread),
            }
        }
    }

    pub struct ThreadPool {
        _size: usize,
        threads: Vec<Worker>,
        sender: Sender<Job>,
    }
    impl ThreadPool {
        pub fn new(size: usize) -> Self {
            assert!(size > 0);

            let mut threads = Vec::with_capacity(size);
            let (sender, receiver) = mpsc::channel(); // new channel
            let receiver = Arc::new(Mutex::new(receiver));

            for id in 0..size {
                threads.push(Worker::new(id, Arc::clone(&receiver)));
            }

            ThreadPool {
                _size: size,
                threads,
                sender,
            }
        }
        pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + 'static + Send,
        {
            // send job to Worker
            let job = Box::new(f);
            self.sender.send(job).unwrap();
        }
    }
    impl Drop for ThreadPool {
        // destory the worker,when thread-pool leave scope.
        fn drop(&mut self) {
            for worker in self.threads.iter_mut() {
                println!("Shutting down worker {}", worker.id);
                if let Some(handle) = worker.thread.take() {
                    handle.join().unwrap();
                }
            }
        }
    }
}

use std::fmt::{Debug, Formatter};
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;

#[derive(Debug)]
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Result<Worker, std::io::Error> {
        let builder = thread::Builder::new();

        let thread = builder.spawn(move || {
            let job = receiver.lock().unwrap().recv().unwrap();

            println!("Worker {id} got a job; executing.");

            job();
        })?;

        Ok(Worker { id, thread })
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct PoolCreationError {
    msg: String,
}

impl Debug for PoolCreationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.msg.as_str())
    }
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Result<ThreadPool, PoolCreationError>
    ///
    /// The `build` function will return PoolCreationError if size is 0.
    /// The function will return PoolCreationError in case of Worker couldn't be created
    pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size == 0 {
            return Err(PoolCreationError { msg: String::from("Pool size has to be greater than 0") });
        }
        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            let worker_result = Worker::new(id, Arc::clone(&receiver));
            if worker_result.is_err() {
                return Err(PoolCreationError { msg: format!("Cannot create pool worker: {:?}", worker_result.unwrap_err()) });
            }
            workers.push(worker_result.unwrap());
        }

        Ok(ThreadPool { workers, sender })
    }

    pub fn new() -> ThreadPool {
        ThreadPool::build(1).unwrap()
    }

    pub fn execute<F>(&self, f: F)
        where F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}

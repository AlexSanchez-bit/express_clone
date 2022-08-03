pub mod thread_pool {
    use std::sync::mpsc::{Receiver, Sender};
    use std::sync::{Arc, Mutex};
    use std::thread::JoinHandle;

    type Job = Box<dyn FnOnce() + Send + 'static>; //task to do in the threads FnOnce is used to free memory after use
    enum Message
//contains a job for the thread to do or terminate if it must stop working
    {
        Do(Job),
        Terminate,
    }
    type Work = Arc<Mutex<Receiver<Message>>>; // a thread safe message

    struct Worker {
        //the thread controller
        id: u32,
        work_flow: JoinHandle<()>,
    }

    impl Worker {
        fn new(id: u32, to_do: Work) -> Worker {//spawn the thread and start working
            use std::thread;
            Worker {
                id,
                work_flow: thread::spawn(move || loop {
                    let pending_work = to_do.lock().unwrap().recv().unwrap(); //get the Message 

                    match pending_work {
                        Message::Do(job) => {
                            //if Message contains a Job move out of it and execute it
                            job();
                        }
                        Message::Terminate => {
                            //end up the loop
                            println!("turninf off the thread {}", id);
                            break;
                        }
                    }
                }),
            }
        }

        fn finish(self) //wait for the thread to finish working 
        {
            let id = self.id;
            self.work_flow.join().unwrap_or_else(|_| {
                println!("error terminando el hilo: {}", id);
            });
        }
    }

   
    pub struct ThreadPool {
        number_of_threads: u16, 
        workers: Vec<Worker>, 
        data_sender: Sender<Message>, 
        data_receiver: Work, 
    }

    impl ThreadPool {
        pub fn new(numb: u16) -> ThreadPool { //creates the thread pool
            let (snd, rv) = std::sync::mpsc::channel(); // create a channel to transfer data between the threads

            ThreadPool {
                number_of_threads: numb,
                workers: Vec::with_capacity(numb as usize),
                data_sender: snd,
                data_receiver: Arc::new(Mutex::new(rv)),
            }

        }

        pub fn initialize(&mut self) {
            for i in 0..self.number_of_threads { //creates the workers and start working 
                self.workers
                    .push(Worker::new(i as u32, Arc::clone(&self.data_receiver))); //
            }
        }

        pub fn send_data<T>(&self, method: T) //sends the messages to the threads
        where
            T: FnOnce() + Send + 'static,
        {
            self.data_sender
                .send(Message::Do(Box::new(method))) //sends the job
                .unwrap_or_else(|err| {
                    print!("fail to send a job :{}", err);
                });
        }
    }
    use std::ops::Drop; //drop implementation for thread pool to release the threads 
    //stops all the workers and release them
    impl Drop for ThreadPool {
        fn drop(&mut self) {
            for _ in 0..self.number_of_threads {
                self.data_sender.send(Message::Terminate).unwrap();//sends Terminate to the workers
            }
            while self.workers.len() > 0 { // while theres still workers
                let aux = self.workers.pop();//remove them from th list
                match aux {
                    Some(worker) => {
                        worker.finish();//if there i a worker wait for him to finish
                    }
                    _ => {}
                }
            }
        }
    }
}

pub mod express {
    use crate::thread_pool::ThreadPool;
    use std::collections::HashMap;
    use std::io::prelude::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::{Arc, Mutex};

    static __DIRNAME: &str = "./";

    type Exception = Box<dyn std::error::Error>;
    type Callback =Box<dyn FnMut(Request,Response) + Send + 'static>;
    type CallbackContainer =HashMap<String, Arc<Mutex<callback_caller>>>;


    struct callback_caller
    {
        callback:Callback
    }

    impl callback_caller
    {

        fn new<T>(cb:T)->callback_caller
            where T:FnMut(Request,Response) +Send +'static
        {
            callback_caller{
                callback:Box::new(cb)
            }
        }

        fn call(&mut self,req:Request,res:Response){
            (*self.callback)(req,res);
        }

    }

   
    pub struct App {
        getter: CallbackContainer,
        setter: CallbackContainer,
        views: String,
        static_folder: String,
        threads: u16,
    }

    impl App {
        pub fn new(thread_number: u16) -> App {
            App {
                getter: HashMap::new(),
                setter: HashMap::new(),
                views: String::from(__DIRNAME),
                static_folder: String::from(__DIRNAME),
                threads: thread_number,
            }
        }

        pub fn static_folder(&mut self, path: &str) {
            self.static_folder = String::from(path);
        }

        pub fn set_views(&mut self, path: &str) {
            self.views = String::from(path);
        }

        pub fn get<T>(&mut self, end_point: &str, callback: T)
        where
            T: FnMut(Request, Response) + Send +Sync+ 'static,
        {
            let callback = Arc::new(Mutex::new(callback_caller::new(callback)));
            self.getter.insert(
                String::from(format!("GET {} HTTP/1.1\r\nHost:", end_point)),
            callback,
            );
        }

        pub fn set<T>(&mut self, end_point: &str, callback: T)
        where
            T: FnMut(Request, Response) + Send +Sync+ 'static,
        {
            let callback = Arc::new(Mutex::new(callback_caller::new(callback)));
            self.setter.insert(
                String::from(format!("GET {} HTTP/1.1\r\n", end_point)),
            callback,
            );
        }

        pub fn listen(&mut self, ip: &str, port: u16) -> Result<bool, Box<dyn std::error::Error>> {
            
            let listenner = TcpListener::bind(format!("{}:{}", ip, port))?;
            let mut threads = ThreadPool::new(self.threads);
            threads.initialize();

            for stream in listenner.incoming() {
                let mut stream = stream.unwrap();

                let default:Callback=Box::new(
                                    |req,mut res:Response|{
                                        res.send("nada que mostrar").unwrap();
                                    }
                    );
                let mut executor=Arc::new(Mutex::new(callback_caller::new(default)));

                let get = App::handle_conection(self.getter.keys(), &mut stream);
                let set = App::handle_conection(self.setter.keys(), &mut stream);

                let mut req = Request::new("", "");
                if let Some(key) = get {
                    req = Request::new(&key.clone(), &key.clone());
                    let meth =self.getter.get(&key).unwrap();                    
                    executor=Arc::clone(meth);
                }else if let Some(key) = set
                {
                    req = Request::new(&key.clone(), &key.clone());
                    let meth =self.setter.get(&key).unwrap();                    
                    executor=Arc::clone(meth);
                }

                let strea = Arc::new(Mutex::new(stream));
                let res = Response::new(Arc::clone(&strea));
                threads.send_data(move || {
                    executor.lock().unwrap().call(req,res);
                });
            }
            drop(self.threads);
            Ok(true)
        }

        fn handle_conection<'a, T>(mut end_points: T, stream: &mut TcpStream) -> Option<String>
        where
            T: Iterator<Item = &'a String>,
        {
            let mut buffer = [0; 512];
            stream.read(&mut buffer).unwrap();
              println!("se hizo req a:{:?} ",String::from_utf8_lossy(&buffer));
            let mut matched = Option::None;
            if end_points.any(|elem| {
                  println!("\n\n y el elemento era: {:?}",elem);
                if buffer.starts_with(elem.as_bytes()) {
                    print!("aqui llego algo");
                    matched = Option::Some(elem.clone());
                    true
                } else {
                    false
                }
            }) {}
            matched
        }

    }

    /*
     *
     * idea de como podria ser un request
     *  app.get("/some/:param1/:param2/:param3",|req,res|{
     *    let param1 = req.params["param1"];
     *    res.send(param1);
     *  });
     *
     * */
    pub struct Request
//object to save the request data
    {
        pub params: HashMap<String, String>,
        original_endpoint: String,
    }

    impl Request {
        fn new(recibed: &str, original: &str) -> Request {
            let mut params_map = HashMap::new();
            let original_params = original.split("/").collect::<Vec<&str>>() as Vec<&str>;
            let recibed_params = recibed.split("/").collect::<Vec<&str>>();

            for i in 0..original_params.len() {
                let aux1 = String::from(&original_params[i][..]);
                let aux2 = String::from(recibed_params[i]);
                params_map.insert(aux1, aux2);
            }

            Request {
                params: params_map,
                original_endpoint: String::from(original),
            }
        }
    }

    pub struct Response
//object to respond and serve data from server
    {
        stream: Arc<Mutex<TcpStream>>,
    }

    impl Response {
        fn new(stream: Arc<Mutex<TcpStream>>) -> Response //constructor
        {
            Response { stream }
        }

        pub fn send_file(&mut self, filepath: &str) -> Result<(), Exception> //send a file
        {
            use std::fs;
            let readed = fs::read(filepath).unwrap();

            let result = String::from_utf8(readed.clone());

            match result {
                Ok(text) => {
                    self.send(&text[..])?;
                }
                Err(_) => {
                    let mut stream = self.stream.lock().unwrap();
                    stream.write(&readed)?;
                    stream.flush().unwrap();
                }
            }

            Ok(())
        }

        pub fn send(&mut self, data: &str) -> Result<(), Exception> //send text
        {
            let status = "HTTP/1.1 200 OK\r\n\r\n";
            let response = format!(" {}{} ", status, data);
            let mut stream = self.stream.lock().unwrap();
            stream.write(response.as_bytes())?;
            stream.flush()?;
            Ok(())
        }
    }
    //traits
}

mod thread_pool {
    use std::sync::mpsc::{Receiver, Sender};
    use std::sync::{Arc, Mutex};
    use std::thread::JoinHandle;

    //workers encargados de realizar las tareas en los hilos
    type Job = Box<dyn FnOnce() + Send + 'static>; //un job es un punteroa un metodo que se llamara en el hilo

    enum Message
//message tiene un Doque contiene un job y un terminate para que el hilo termine su trabajo
    {
        Do(Job),
        Terminate,
    }
    type Work = Arc<Mutex<Receiver<Message>>>; //work es un trabajo a realizar por cada hilo

    struct Worker {
        //estructura del worker
        id: u32,
        work_flow: JoinHandle<()>,
    }

    impl Worker {
        fn new(id: u32, to_do: Work) -> Worker {
            use std::thread;
            Worker {
                id,
                work_flow: thread::spawn(move || loop {
                    let pending_work = to_do.lock().unwrap().recv().unwrap(); //saca el message del Work

                    match pending_work {
                        Message::Do(job) => {
                            //si se recibe un trabajo ejecutalo
                            job();
                        }
                        Message::Terminate => {
                            //si se termino rompe el bucle
                            println!("cancelando hilo {}", id);
                            break;
                        }
                    }
                }),
            }
        }

        fn finish(self) //espra a que los hilos terminen de ejecutar
        {
            let id = self.id;
            self.work_flow.join().unwrap_or_else(|_| {
                println!("error terminando el hilo: {}", id);
            });
        }
    }

    //estructura que contiene el thread_pool

    pub struct ThreadPool {
        number_of_threads: u16,
        workers: Vec<Worker>,
        data_sender: Sender<Message>,
        data_receiver: Work,
    }

    impl ThreadPool {
        pub fn new(numb: u16) -> ThreadPool {
            let (snd, rv) = std::sync::mpsc::channel();
            ThreadPool {
                number_of_threads: numb,
                workers: Vec::with_capacity(numb as usize),
                data_sender: snd,
                data_receiver: Arc::new(Mutex::new(rv)),
            }
        }

        pub fn initialize(&mut self) {
            for i in 0..self.number_of_threads {
                self.workers
                    .push(Worker::new(i as u32, Arc::clone(&self.data_receiver)));
            }
        }

        pub fn send_data<T>(&self, method: T)
        where
            T: FnOnce() + Send + 'static,
        {
            self.data_sender
                .send(Message::Do(Box::new(method)))
                .unwrap_or_else(|err| {
                    print!("fallo al enviar un trabajo :{}", err);
                });
        }
    }
    use std::ops::Drop;

    impl Drop for ThreadPool {
        fn drop(&mut self) {
            for _ in 0..self.number_of_threads {
                self.data_sender.send(Message::Terminate).unwrap();
            }
            while self.workers.len() > 0 {
                let aux = self.workers.pop();
                match aux {
                    Some(worker) => {
                        worker.finish();
                    }
                    _ => {}
                }
            }
        }
    }
}

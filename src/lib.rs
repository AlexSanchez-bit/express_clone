pub mod Express
{
    use std::net::{TcpStream,TcpListener};
    use std::io::prelude::{Read,Write};
    use std::sync::{Arc,Mutex};
 
   type Exception =Box<dyn std::error::Error>;

    pub struct App
    {
        getter:Vec<String>,
        setter:Vec<String>,
        threads:u16
    }


    impl App{
        pub fn new(thread_number:u16)->App
        {
            App{
                getter:vec![],
                setter:vec![],
                threads:thread_number
            }
        }

        pub fn get(&mut self,end_point:&str){
            self.getter.push(String::from(format!("HTTP/1.1 {} 200\r\n\r\n", end_point)));
        }
        pub fn set(&mut self,end_point:&str){
            self.setter.push(String::from(end_point));
        }

        pub fn listen(&mut self,ip:&str,port:u16)->Result<bool,Box<dyn std::error::Error>>
        {
            let listenner = TcpListener::bind(format!("{}:{}",ip,port))?;
            let mut threads = crate::thread_pool::ThreadPool::new(self.threads);
            threads.initialize();            

            for stream in listenner.incoming()
            {
                let stream = stream.unwrap();
                let end_points = self.getter.clone();
                threads.send_data(move ||{
                    App::handle_conection(end_points,stream);
                });
            }

                  Ok(true)
        }

        fn handle_conection(_end_points:Vec<String>,mut stream:TcpStream)
        {
            let mut buffer =[0u8;512];
            stream.read(&mut buffer).unwrap();
            print!("se hizo request a :{} ",String::from_utf8_lossy(&buffer));
            let stream= Arc::new(Mutex::new(stream));
             let mut req = Response::new(Arc::clone(&stream));           
             //req.send("se respondio del servidor");
             req.send_file("/home/nadie/datos/telegram/ProyectoNodejs/Museo/src/views/colaborar.html").unwrap();
        }
    }


    pub struct Response
    {
        stream:Arc<Mutex<TcpStream>>,        
    }

    impl Response
    {
        pub fn new(stream:Arc<Mutex<TcpStream>>)->Response
        {
            Response
            {
                stream
            }
        }

        pub fn send_file(&mut self,filepath:&str)->Result<(),Exception>
        {
            use std::fs;
           let readed = fs::read(filepath).unwrap();

            let result = String::from_utf8(readed.clone());

            match result
            {
                Ok(text)=>
                {
                self.send(&text[..])?;
                },
                Err(_)=>
                {
           let mut stream = self.stream.lock().unwrap();
           stream.write(&readed)?;
           stream.flush().unwrap();
                }
            }

           Ok(())
        }


        pub fn send(&mut self,data:&str)->Result<(),Exception>
        {
            let status="HTTP/1.1 200 OK\r\n\r\n";
            let response = format!("{}{}",status,data);
             let mut stream=self.stream.lock().unwrap();
             stream.write(response.as_bytes())?;
             stream.flush()?;
           Ok(()) 
        }

    }

}


mod thread_pool
{
    use std::thread::JoinHandle;
    use std::sync::mpsc::{Receiver,Sender};
    use std::sync::{Arc,Mutex};    

    //workers encargados de realizar las tareas en los hilos
    type Job = Box<dyn FnOnce() + Send +'static>;//un job es un punteroa un metodo que se llamara en el hilo

    enum Message//message tiene un Doque contiene un job y un terminate para que el hilo termine su trabajo
    {
        Do(Job),
        Terminate
    }
    type Work = Arc<Mutex<Receiver<Message>>>;//work es un trabajo a realizar por cada hilo

    struct Worker{//estructura del worker
        id:u32,
        work_flow:JoinHandle<()>
    }

    impl Worker
    {
         fn new(id:u32,to_do:Work)->Worker
        {
    use std::thread;
            Worker{
                id,
                work_flow:thread::spawn(move || loop
                    {
                            let pending_work= to_do.lock().unwrap().recv().unwrap();//saca el message del Work

                            match pending_work
                            {
                                Message::Do(job)=>{//si se recibe un trabajo ejecutalo
                                    job();
                                },
                                Message::Terminate=>{//si se termino rompe el bucle
                                    println!("cancelando hilo {}",id);
                                    break;
                                }
                            }
                    }),
            }
        }

         fn finish(self)//espra a que los hilos terminen de ejecutar 
        {
            let id = self.id;
            self.work_flow.join().unwrap_or_else(|_|{
                println!("error terminando el hilo: {}",id);
            });        
        }
    }


    //estructura que contiene el thread_pool

    pub struct ThreadPool
    {        
        number_of_threads:u16,
        workers:Vec<Worker>,
        data_sender:Sender<Message>,
        data_receiver:Work,
    }

    impl ThreadPool
    {

        pub fn new(numb:u16)->ThreadPool
        {
            let (snd,rv) = std::sync::mpsc::channel();
            ThreadPool{
                number_of_threads:numb,
                workers:Vec::with_capacity(numb as usize),
                    data_sender:snd,
                    data_receiver:Arc::new(Mutex::new(rv))
            }
        }

        pub fn initialize(&mut self)
        {
            for i in 0..self.number_of_threads
            {
                self.workers.push(Worker::new(i as u32,Arc::clone(&self.data_receiver)));
            }
        }
    

        pub fn send_data<T>(&self,method:T) where T :FnOnce() + Send +'static
        {
            self.data_sender.send(Message::Do(Box::new(method))).unwrap_or_else(|err|{
                print!("fallo al enviar un trabajo :{}",err);
            });
        } 
    }
        use std::ops::Drop;

        impl Drop for ThreadPool{

             fn drop(&mut self){
                    for _ in 0..self.number_of_threads                    {                    
                        self.data_sender.send(Message::Terminate).unwrap();
                    }
                    while self.workers.len() >0
                    {
                        let aux = self.workers.pop();
                        match aux{
                            Some(worker)=>{
                                worker.finish();
                            },
                            _=>{}
                        }
                    }
            }

        }


}
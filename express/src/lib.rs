pub mod express {
    use thread_pool::thread_pool::ThreadPool;
    use std::collections::HashMap;
    use std::io::prelude::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::{Arc, Mutex};

    type Exception = Box<dyn std::error::Error>; //to manage exceptions
    type CallbackContainer = HashMap<String, Arc<Mutex<CallbackCaller>>>; //callback container 
    type Callback = Box<dyn FnMut(Request, Response) + Send + 'static>; //traitobject to store callback

    struct CallbackCaller {
        //allow to call the callbacks without drop the boxes
        callback: Callback,
    }

    impl CallbackCaller {
        fn new<T>(cb: T) -> CallbackCaller
        where
            T: FnMut(Request, Response) + Send + 'static,
        {
            //storage the callback 
            CallbackCaller {
                callback: Box::new(cb),
            }
        }

        fn call(&mut self, req: Request, res: Response) {
            (*self.callback)(req, res);
            //call the methos without drop them 
        }
    }

    enum HTTPSTATUS { //status of the requests
        GET,
        POST,
        FILE,
        NOTFOUND,
    }

    pub struct App { //app manages and storage the whole page
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
                views: std::env::args().nth(0).unwrap(),//by default will be the original path to the app
                static_folder: std::env::args().nth(0).unwrap(),
                threads: thread_number,
            }
        }

        pub fn static_folder(&mut self, path: &str) { //set the static folder to get the files
            self.static_folder = String::from(path);
        }

        pub fn set_views(&mut self, path: &str) {//set the view container for the render
            self.views = String::from(path);
        }

        pub fn get<T>(&mut self, end_point: &str, callback: T)
        where
            T: FnMut(Request, Response) + Send + Sync + 'static,
        {
            //add  the get endpoints
            let callback = Arc::new(Mutex::new(CallbackCaller::new(callback)));
            self.getter
                .insert(String::from(format!("{}", end_point.trim())), callback);
        }

        pub fn post<T>(&mut self, end_point: &str, callback: T)
        where
            T: FnMut(Request, Response) + Send + Sync + 'static,
        {//add the post endpoints
            let callback = Arc::new(Mutex::new(CallbackCaller::new(callback)));
            self.setter
                .insert(String::from(format!("{}", end_point.trim())), callback);
        }

        pub fn listen(&mut self, ip: &str, port: u16) -> Result<bool, Box<dyn std::error::Error>> {
            //starts the server at the ip and port
            let listenner = TcpListener::bind(format!("{}:{}", ip, port))?;//start listenning the HTTP server
            let mut threads = ThreadPool::new(self.threads);//creates the thread pool
            threads.initialize();//initialize the thread pool

            for stream in listenner.incoming() 
            { //iterates over the incoming http requests (its lazy so it will run till server is down)
                let mut buffer = [0; 516]; //buffer to read/write the requests
                let mut stream = stream.unwrap(); //get the stream to manage the http request
                stream.read(&mut buffer).unwrap(); //read the bytes to the buffer

                let default: Callback = Box::new(|_req, mut res| {
                    res.send(" noting here to show ").unwrap();
                }); //default answer

                //creating request and response objects
                let strea = Arc::new(Mutex::new(stream));
                let res = Response::new(Arc::clone(&strea), self.views.clone());

                let mut executor = Arc::new(Mutex::new(CallbackCaller::new(default))); //callback executor

                let (status,endp) = App::handle_conection(buffer); //reads the data storaged in the buffer and returns the status and endpoint

                let mut req = Request::new("", "");// creating the request
                match status {  //matching the status
                    HTTPSTATUS::GET => {
                        let cb = self.getter.get(&endp);

                        match cb {
                            Some(_cb) => {
                                req = Request::new(&endp, &endp);
                                executor = Arc::clone(_cb);
                                //if the request is on the container just set the req and the
                                //executor to the order
                            }
                            None => {
                                //else first see if any arg based endpoint matches the request
                                for key in self.getter.keys() {
                                    if App::match_endp_params(key, &endp) {
                                        //if so set the variables
                                        req = Request::new(&endp, key);
                                        executor = Arc::clone(self.getter.get(key).unwrap());
                                        break;
                                    }
                                }
                                //else just throw the default answer
                            }
                        }
                    }
                    HTTPSTATUS::POST => {
                        //same as GET
                        req = Request::new(&endp, &endp);
                        let cb = self.setter.get(&endp);

                        match cb {
                            Some(_cb) => {
                                executor = Arc::clone(_cb);
                            }
                            _ => {}
                        }
                    }
                    HTTPSTATUS::FILE => {
                        //for FILE sending just try to read and show the message else
                        let st_folder = self.static_folder.clone();
                        let file_response: Callback = Box::new(move |_req, mut res| {
                            res.send_file(&format!("{}/{}", st_folder, &endp))
                                .unwrap_or_else(|_| {
                                    println!("file not found {}", endp);
                                });
                        });
                        executor = Arc::new(Mutex::new(CallbackCaller::new(file_response)));
                    }
                    _ => {}
                }

                threads.send_data(move || {
                    executor.lock().unwrap().call(req, res);//manages the endpoint within the threadpool
                });
            }
            drop(self.threads);//drop threadpool
            Ok(true)
        }

        fn match_endp_params(with_params: &str, withouth: &str) -> bool {
            //true if an parametrized endpoint match the endpoint
            let params = with_params.split("/");
            let original = withouth.split("/");

            if params.clone().count() != original.clone().count() {
                return false;
            }

            for (key1, key2) in params.zip(original) {
                if key1 == key2 && key1!="" {
                    return true;
                }
            }
            false
        }

        fn handle_conection(buffer: [u8; 516]) -> (HTTPSTATUS, String) {
            //handles the connection and return HTTPSTATUS and endpoint if matches
            let endp_name = App::chop_input(buffer).trim().to_string();
            let mut ret = (HTTPSTATUS::NOTFOUND, endp_name);

            if App::is_file(&ret.1) {
                ret.0 = HTTPSTATUS::FILE;
            } else if buffer.starts_with(b"POST") {
                ret.0 = HTTPSTATUS::POST;
            } else if buffer.starts_with(b"GET") {
                ret.0 = HTTPSTATUS::GET;
            }

            ret
        }

        fn is_file(enp_name: &String) -> bool {
            //return if the endpoint references a file
            enp_name.contains(".")
        }
        fn chop_input(buffer: [u8; 516]) -> String {
            //return the actual endpoint contained in the buffer
            let string = String::from_utf8_lossy(&buffer).to_string();
            let mut i = 0;
            let mut init = 0;

            for letter in string.chars() {
                if init == 0 && letter == ' ' {
                    init = i;
                }

                if letter == 'H' {
                    return String::from(&string[init..i]);
                }
                i += 1;
            }
            string
        }
    }

    /*
     *
     * how a request could look like
     *  app.get("/some/:param1/:param2/:param3",|req,res|{
     *    let param1 = req.params["param1"];
     *    res.send(param1);
     *  });
     *
     * */

    pub enum Data {
        STRING(String),
        INT(i64),
        FLOAT(f64),
        UNDEFINED,
    }
    pub struct Request
//object to save the request data
    {
        pub params: HashMap<String, Data>,
        _original_endpoint: String,
    }

    impl Request {
        fn new(recibed: &str, original: &str) -> Request {
            //get the url parameters values and save them 
            let mut params_map = HashMap::new();
            let original_params = original.split("/").collect::<Vec<&str>>() as Vec<&str>;
            let recibed_params = recibed.split("/").collect::<Vec<&str>>();

            for i in 0..original_params.len() {
                let aux1 = String::from(&original_params[i][..]);
                let aux2 = String::from(recibed_params[i]);

                if aux1.contains(":") {

                    //tries to check datatypes and return them in a Data enum
                    let mut data=Data::STRING(aux2.clone());                    
                    let int = aux2.parse::<i64>();
                    let float = aux2.parse::<f64>();

                    if int.is_ok()
                    {
                        data=Data::INT(int.unwrap()); 
                    }
                    if float.is_ok()
                    {
                        data=Data::FLOAT(float.unwrap()); 
                    }

                    params_map.insert(String::from(&aux1[1..]),data);
                }
            }

            Request {
                params: params_map,
                _original_endpoint: String::from(original),
            }
        }

        pub fn get_param(&mut self, param_name: &str) -> Option<Data> {
            //returns the value of the url param behind the Data enum
            self.params.remove(param_name)
        }
    }

    pub struct Response
        //object to describe the response
    {
        stream: Arc<Mutex<TcpStream>>,
        view_directory: String,
    }

    impl Response {
        fn new(stream: Arc<Mutex<TcpStream>>, view_directory: String) -> Response //constructor
            //stream to manage the http response
        {
            Response {
                stream,
                view_directory,
            }
        }

        pub fn render(&mut self, filename: &str) -> Result<(), Exception> {
            //renders the page based on the configured render //last thing still a job in progress
            let format = if filename.ends_with(".html") {
                ""
            } else {
                ".html"
            };
            let final_archive = format!(
                "{}{}{}{}",
                self.view_directory,
                if filename.starts_with("/") { "" } else { "/" },
                filename,
                format
            );
            self.send_file(&final_archive)?;
            Ok(())
        }

        pub fn send_file(&mut self, filepath: &str) -> Result<(), Exception> //send a file
        {
            //sends a file to the client
            use std::fs;
            let readed = fs::read(filepath)?;

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
            //send a text to the client
            let status = "HTTP/1.1 200 OK\r\n\r\n";
            let response = format!(" {}{} ", status, data);
            let mut stream = self.stream.lock().unwrap();
            stream.write(response.as_bytes())?;
            stream.flush()?;
            Ok(())
        }
    }
}

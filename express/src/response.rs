
    use std::net::TcpStream;
    use std::sync::{Arc,Mutex};
    use std::io::prelude::Write;

    type Exception =Box<dyn std::error::Error>;

pub struct Response
        //object to describe the response
    {
        stream: Arc<Mutex<TcpStream>>,
        view_directory: String,
    }

    impl Response {
       pub fn new(stream: Arc<Mutex<TcpStream>>, view_directory: String) -> Response //constructor
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

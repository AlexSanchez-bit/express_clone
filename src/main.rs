use express::express::{App,Data};
fn main() {
    //-----------------
    let mut app = App::new(4);//parameter number of threads

   // app.static_folder(""); //default
   // "./"
   // app.set_views(""); //default "./"

    app.get("/", |_req, mut res| {
        res.render("/index").unwrap();//render a file acording to the configured render (html) default
        res.send_file("path/to/file").unwrap();//sends a file to the client
    });
    app.get("/home/:param1", |mut req, mut res| {
        let param = req.get_param("param1").unwrap();//get the parameter by name        
        match param 
        {
            Data::STRING(i)=>{
                println!("{}",i);
            }
            _=>{}
        }
        res.send("respondido desde el home").unwrap(); //send a text to the client
    });

    app.post("/", |_req, _res| { //post endpoint
        println!("se hizo post ");
    });

    app.listen("127.0.0.1", 8080).unwrap();
}

use express_clone::express::App;
fn main() {

    let alg = b"GET / HTTP/1.1\r\n";
    let other = String::from("GET / HTTP/").as_bytes();

    

    //-----------------
    let mut app = App::new(4);
    app.get("/", |_req, mut res| {
        res.send_file("/home/nadie/datos/telegram/ProyectoNodejs/Pagina/src/views/index.html").unwrap();
    });
    app.get("/home", |_req, mut res| {
        res.send("respondido desde el home").unwrap();
    });

    app.set("/post_data",|_req,_res|{
        println!("se hizo post ");
    });

    app.listen("127.0.0.1", 8080).unwrap();
}


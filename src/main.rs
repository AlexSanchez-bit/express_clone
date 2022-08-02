use express_clone::express::App;
fn main() {
    //-----------------
    let mut app = App::new(4);
    app.get("/", |_req, mut res| {
        res.send_file("/home/nadie/datos/telegram/ProyectoNodejs/Pagina/src/views/index.html")
            .unwrap();
    });
    app.get("/home", |_req, mut res| {
        std::thread::sleep(std::time::Duration::from_secs(10));
        res.send("respondido desde el home").unwrap();
    });

    app.set("/post_data", |_req, _res| {
        println!("se hizo post ");
    });

    app.listen("127.0.0.1", 8080).unwrap();
}

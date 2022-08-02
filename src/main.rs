use express_clone::express::App;
fn main() {
    //-----------------
    let mut app = App::new(4);

    app.static_folder("/home/nadie/datos/telegram/ProyectoNodejs/Pagina/src/public");
    app.set_views("/home/nadie/datos/telegram/ProyectoNodejs/Pagina/src/views");

    app.get("/", |_req, mut res| {
        res.render("/index.html").unwrap();
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

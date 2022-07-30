use express_clone::express::App;
fn main() {

    let mut app = App::new(4);
    app.get("/",|req,mut res|{
        res.send("respondido desde el server").unwrap();
    });
    app.get("/home",|req,mut res|{
        res.send("respondido desde el server").unwrap();
    });
    app.listen("127.0.0.1",8080).unwrap();

}

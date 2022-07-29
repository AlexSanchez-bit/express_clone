use express_clone::Express::App;
fn main() {

    let mut app = App::new(4);
    app.get("/");
    app.get("/home");
    app.get("/endp1");
    app.get("/endp2");
    app.listen("127.0.0.1",8080);

}

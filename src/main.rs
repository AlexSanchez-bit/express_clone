use express_clone::express::App;
use express_clone::express::Data;
fn main() {
    //-----------------
    let mut app = App::new(4);

    app.static_folder("/home/nadie/datos/telegram/ProyectoNodejs/Pagina/src/public");
    app.set_views("/home/nadie/datos/telegram/ProyectoNodejs/Pagina/src/views");

    app.get("/", |_req, mut res| {
        res.render("/index.html").unwrap();
    });
    app.get("/home/:param1", |mut req, mut res| {
        let param = req.get_param("param1").unwrap();
        match param
        {
            Data::STRING(i)=>{
                println!("{}",i);
            }
            _=>{}
        }
        res.send("respondido desde el home").unwrap();
    });

    app.post("/", |_req, _res| {
        println!("se hizo post ");
    });

    app.listen("127.0.0.1", 8080).unwrap();
}

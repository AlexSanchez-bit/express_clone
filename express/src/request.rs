    use std::collections::HashMap;
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
       pub fn new(recibed: &str, original: &str) -> Request {
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

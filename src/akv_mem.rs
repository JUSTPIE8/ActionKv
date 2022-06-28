use libactionkv::ActionKV;

//only applicable for linux for now

const USAGE: &str = "
Usage:
    akv_mem File get Key
    akv_mem File delete Key
    akv_mem File insert Key Value
    akv_mem File update Key Value
";

fn main() {
    //for getting comamnd line arguments
    let args: Vec<String> = std::env::args().collect();
    let fname = args.get(1).expect(&USAGE);
    let action: &str = args.get(2).expect(&USAGE).as_ref();
    let key: &str = args.get(3).expect(&USAGE).as_ref();
    let maybe_value = args.get(4);
    //  println!("{:?}", action);

    let path = std::path::Path::new(&fname);

    let mut store = ActionKV.open(path).expect("unable to open a file ");
    store.load().expect("unable to load data");

    match action {
        "get" => match store.get(key).unwrap() {
            None => eprintln!("{:?} not found ", key),
            Some(value) => println!("{:?}", value),
        },
        "delete" => store.delete(key).unwrap(),
        "insert" => {
            let value = maybe_value.expect(&USAGE).as_ref();
            store.insert(key, value).unwrap()
        }
        "update" => {
            let value = maybe_value.expect(&USAGE).as_ref();
            store.update(key, value).unwrap()
        }
        _ => eprintln!("{}", &USAGE),
    }
}

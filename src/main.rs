use rutag::{get_xattr, list_xattrs, set_xattr};

fn main() {
    let mut args = std::env::args().into_iter().skip(1);

    let op = args.next().expect("operation");
    let filename = args.next().expect("filename");

    match &op[..] {
        "get" => {
            let key = args.next().expect("key");
            match get_xattr(&filename, key) {
                Ok(tag) => println!("{} - {}", filename, tag),
                Err(e) => eprintln!("{}", e),
            }
        }
        "list" => match list_xattrs(&filename) {
            Ok(attrs) => println!("{:#?}", attrs),
            Err(e) => eprintln!("{}", e),
        },
        "set" => {
            let key = args.next().expect("key");
            let value = args.next().expect("value");

            if let Err(e) = set_xattr(filename, key, value) {
                eprintln!("{}", e);
            }
        }
        _ => {}
    }
}

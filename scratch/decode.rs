use bson::Document;
use std::io::Cursor;

fn main() {
    let hex_str = "1a00000002494400050000004b45727200104552000100000000";
    let bytes = hex::decode(hex_str).unwrap();
    let doc = Document::from_reader(&mut Cursor::new(bytes)).unwrap();
    println!("{:?}", doc);
}

use prost::Message;
use try_prost::pb::*;

fn main() {
    // let person = Person::default();
    let phones = vec![PhoneNumber::new("111-333-4444", PhoneType::Mobile)];
    let person = Person::new("Frankorz",1,"superfrankie621@gmail.com", phones);
    let v1 = person.encode_to_vec();
    #[allow(unused_variables)]
    let v2 = person.encode_length_delimited_to_vec();

    let person_decode = Person::decode(v1.as_ref()).unwrap();

    assert_eq!(&person, &person_decode);
    // println!("{person:?}, {v1:?}(len: {}), {v2:?}", v1.len());

    let json = serde_json::to_string_pretty(&person_decode).unwrap();
    println!("{}", json);
}

fn main() {
    let arr1 = [1, 2, 3];
    let arr2 = [1, 2, 3, 4, 5];
}
fn get_type_of<T>(_: &T) -> &str {
    std::any::type_name::<T>()
}

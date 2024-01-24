fn main() {
    println!("Hello, world!");
}

#[allow(non_snake_case)]
pub extern "C" fn CorePluginInit() -> bool {
    main();
    true
}

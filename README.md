# Bakery

A Rust library to deserialize binary objects from structured text data files.

## Basic example

    use bakery::load_struct_from_string;
    use bakery_derive::Recipe;
    use serde::Deserialize;
    
    #[derive(Recipe, Deserialize)]
    struct GameConfig {
        width: u32,
        height: u32,
        fullscreen: bool
    }
    
    let config: GameConfig = load_struct_from_string("width: 1024, height: 768, fullscreen: true");

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

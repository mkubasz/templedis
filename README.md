# Templedis

Templedis is an implementation of the Redis key-value store in Rust. It is a fully asynchronous, non-blocking library that is designed to be efficient and scalable.

### Features

- Full Redis protocol support
- Asynchronous, non-blocking operation
- High performance and scalability
- Easy to use API

### Installation
Templedis is available on crates.io. To install it, run the following command:

`cargo install templedis`

### Usage
To use Templedis, you will need to create a Templedis client. Once you have a client, you can start using the Redis commands. For example, to set a key-value pair, you would use the following code:
```rust
let mut client = Templedis::new();
client.set("key", "value");
```

## Documentation

The full documentation for Templedis is available on [the project website](https://templedis.rs/).

## License

Templedis is licensed under the MIT License.

## Author

Templedis was created by Mateusz Kubaszek.

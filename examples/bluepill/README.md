# Example: read_all

This Blue Pill board example reads all 16 (or 8 with minor changes) channels on a 4067 (or 4051) series analog multiplexer every 100ms and pretty-prints the results over an ST-LINK debugger via probe-rs (`rprintln!()`).

# Usage

Here's an imaginary example using a 74HC4067 with a Blue Pill (stm32f104) board...

```shell
$ cargo run --example read_all
```

Here's me using it to read a hall effect sensor on channel 15:

![Reading a hall effect sensor on channel 15](https://thumbs.gfycat.com/FlippantAptHadrosaurus-size_restricted.gif)

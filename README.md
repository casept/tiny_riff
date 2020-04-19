# tiny_riff

This library aims to provide (currently planned to be read-only) support for the [RIFF file format](https://en.wikipedia.org/wiki/Resource_Interchange_File_Format).

It's primarily designed for bare-metal embedded environments (the primary target is the GBA, for loading assets in a game I'm making). Therefore, `std::io` APIs are not used (if you can use them, consider using [this crate](https://github.com/frabert/riff) instead, which also supports writing).

The API is very simplistic. You supply a slice which contains the data, and get back a `RiffReader` which you can then extract chunks out of, either by name or by iterating over each chunk. 
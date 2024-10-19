# Huffman Compression in Rust

This project is a Rust implementation of Huffman compression and decompression, a lossless data compression algorithm. The algorithm builds a binary tree to assign variable-length binary codes to characters based on their frequencies, ensuring that more frequent characters have shorter codes. This project supports compressing text files into a custom binary format and decompressing them back to their original form.

## Features

- Efficient file compression using Huffman encoding.
- Custom file format for storing compressed files.
- Command-line interface (CLI) to compress and decompress files.
- Handles any file type that can be read as binary data.
- Memory safety and error handling using Rust’s powerful type system.

## Installation

### Prerequisites

You must have Rust installed on your machine. If you haven't installed Rust yet, follow the instructions on the [official Rust website](https://www.rust-lang.org/tools/install).

### Clone the Repository

```bash
git clone https://github.com/yourusername/huffman_compression.git
cd huffman_compression
```

### Build the Project
Once you’ve cloned the project, build it using Cargo:
```bash
cargo build --release

```

### Usage 
The program provides two main functionalities: `compress` and `decompress`. Both can be accessed via the command line.

#### Compress a File
To compress a file, run:

```bash
cargo run --release compress <input_file> <output_file>
```

Example:
```bash
cargo run --release compress input.txt compressed.huff
```

This command will take the content of `input.txt`, compress it using Huffman encoding, and save it to `compressed.huff`.

### Decompress a File

To decompress a file, run:
```bash
cargo run --release decompress <input_file> <output_file>
```

Example:
```bash
cargo run --release decompress compressed.huff output.txt
```

This command will take the compressed file `compressed.huff` and decompress it back to its original form, saving it to `output.txt`.

### Project Structure

```bash
ruffman/
│
├── Cargo.toml           # Dependency and project configuration
├── README.md            # Project documentation
└── src
    └── main.rs          # Main logic for compression and decompression
```

#### Key Components
- **Huffman Tree**: The core of the algorithm, used to assign variable-length binary codes to characters based on their frequency.
- **File I/O**: Reading and writing files efficiently using `BufReader` and `BufWriter`.
- **Binary Heap**: Used as a priority queue to build the Huffman tree.


### How It Works
1. **Frequency Table**: The program scans the input file to build a frequency table of characters.
2. **Huffman Tree**: A binary tree is constructed using the frequency table, where each character is a leaf node.
3. **Encoding**: The program generates binary codes for each character based on its position in the Huffman tree.
4. **Compression**: The input file is converted into a bitstream using the Huffman codes and written to the output file.
5. **Decompression**: The encoded bitstream is decoded by reconstructing the Huffman tree and converting the bits back into the original characters.

### Examples

#### Compressing
```bash
cargo run --release compress example.txt compressed.huff
```

#### Decompressing
```bash
cargo run --release decompress compressed.huff decompressed.txt
```

### Limitations
- This project handles plain-text and binary files, but the file size is limited by system memory for in-memory operations.
- It does not yet support multi-threading for larger file compression.

### Future Improvements
- Add multi-threading support for faster compression/decompression of large files.
- Implement more file format support and metadata storage.
- Optimize file read and write performance for larger datasets.

### Contributing
Contributions are welcome! If you want to contribute to the project, feel free to open issues or submit pull requests.

### License
This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.



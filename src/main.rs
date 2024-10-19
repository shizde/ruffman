use std::collections::{BinaryHeap, HashMap};
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::cmp::Ordering;
use std::path::Path;

// Define a node in the Huffman tree
#[derive(Debug, Eq, PartialEq)]
struct Node {
    freq: usize,
    char_val: Option<u8>,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Node {
    fn new(freq: usize, char_val: Option<u8>, left: Option<Box<Node>>, right: Option<Box<Node>>) -> Self {
        Node {
            freq,
            char_val,
            left,
            right,
        }
    }
}

// Custom ordering to make BinaryHeap a min-heap
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.freq.cmp(&self.freq)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Build a frequency table for characters in the input
fn build_frequency_table(data: &[u8]) -> HashMap<u8, usize> {
    let mut frequency = HashMap::new();
    for &byte in data {
        *frequency.entry(byte).or_insert(0) += 1;
    }
    frequency
}

// Build a Huffman tree using the frequency table
fn build_huffman_tree(frequency: HashMap<u8, usize>) -> Option<Box<Node>> {
    let mut heap = BinaryHeap::new();

    for (byte, freq) in frequency {
        heap.push(Box::new(Node::new(freq, Some(byte), None, None)));
    }

    while heap.len() > 1 {
        let left = heap.pop().unwrap();
        let right = heap.pop().unwrap();

        let combined_freq = left.freq + right.freq;
        let new_node = Node::new(combined_freq, None, Some(left), Some(right));

        heap.push(Box::new(new_node));
    }

    heap.pop()
}

// Generate Huffman codes from the Huffman tree
fn generate_codes(node: &Option<Box<Node>>, prefix: String, codes: &mut HashMap<u8, String>) {
    if let Some(n) = node {
        if let Some(c) = n.char_val {
            codes.insert(c, prefix);
        } else {
            generate_codes(&n.left, format!("{}0", prefix), codes);
            generate_codes(&n.right, format!("{}1", prefix), codes);
        }
    }
}

// Compress a file using Huffman encoding
fn compress_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let input_file = File::open(input_path)?;
    let mut reader = BufReader::new(input_file);
    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;

    let frequency_table = build_frequency_table(&data);
    let huffman_tree = build_huffman_tree(frequency_table).unwrap();

    let mut codes = HashMap::new();
    generate_codes(&Some(huffman_tree), String::new(), &mut codes);

    let mut compressed_bits = String::new();
    for byte in data {
        compressed_bits.push_str(&codes[&byte]);
    }

    let mut output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);

    // Write frequency table to the output for decompression purposes
    writer.write_all(&serde_json::to_vec(&codes).unwrap())?;

    // Write compressed bits to the output file as bytes
    let compressed_bytes = convert_bits_to_bytes(&compressed_bits);
    writer.write_all(&compressed_bytes)?;

    Ok(())
}

// Decompress a file using Huffman encoding
fn decompress_file(input_path: &Path, output_path: &Path) -> io::Result<()> {
    let input_file = File::open(input_path)?;
    let mut reader = BufReader::new(input_file);
    let mut compressed_data = Vec::new();
    reader.read_to_end(&mut compressed_data)?;

    // Deserialize the frequency table from the compressed data
    let codes: HashMap<String, u8> = serde_json::from_slice(&compressed_data[..compressed_data.len() - 8]).unwrap();

    // Convert the compressed bits back to binary
    let compressed_bits = convert_bytes_to_bits(&compressed_data[compressed_data.len() - 8..]);

    // Reconstruct the original data using the Huffman codes
    let mut decompressed_data = Vec::new();
    let mut temp_code = String::new();

    for bit in compressed_bits.chars() {
        temp_code.push(bit);
        if let Some(byte) = codes.get(&temp_code) {
            decompressed_data.push(*byte);
            temp_code.clear();
        }
    }

    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    writer.write_all(&decompressed_data)?;

    Ok(())
}

// Convert a string of bits to bytes
fn convert_bits_to_bytes(bits: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    for chunk in bits.as_bytes().chunks(8) {
        let mut byte = 0;
        for &bit in chunk {
            byte = (byte << 1) | (bit - b'0');
        }
        bytes.push(byte);
    }
    bytes
}

// Convert a byte slice to a string of bits
fn convert_bytes_to_bits(bytes: &[u8]) -> String {
    let mut bits = String::new();
    for &byte in bytes {
        bits.push_str(&format!("{:08b}", byte));
    }
    bits
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <compress|decompress> <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }

    let command = &args[1];
    let input_path = Path::new(&args[2]);
    let output_path = Path::new(&args[3]);

    match command.as_str() {
        "compress" => {
            if let Err(e) = compress_file(input_path, output_path) {
                eprintln!("Error compressing file: {}", e);
            }
        }
        "decompress" => {
            if let Err(e) = decompress_file(input_path, output_path) {
                eprintln!("Error decompressing file: {}", e);
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            std::process::exit(1);
        }
    }
}


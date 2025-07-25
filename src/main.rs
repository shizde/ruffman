use std::collections::{BinaryHeap, HashMap};
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::cmp::Ordering;
use std::path::Path;
use serde::{Serialize, Deserialize};

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
        // First compare by frequency, then by character value for deterministic ordering
        other.freq.cmp(&self.freq)
            .then_with(|| self.char_val.cmp(&other.char_val))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Serializable structure for the compressed file header
#[derive(Serialize, Deserialize)]
struct CompressionHeader {
    codes: HashMap<u8, String>,
    original_bit_count: usize, // Track original bit length to handle padding
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
    if frequency.is_empty() {
        return None;
    }
    
    let mut heap = BinaryHeap::new();

    // Handle single character case
    if frequency.len() == 1 {
        let (byte, freq) = frequency.into_iter().next().unwrap();
        return Some(Box::new(Node::new(freq, Some(byte), None, None)));
    }

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
            // Handle single character case - assign a default code
            let code = if prefix.is_empty() { "0".to_string() } else { prefix };
            codes.insert(c, code);
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

    if data.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Input file is empty"));
    }

    let frequency_table = build_frequency_table(&data);
    let huffman_tree = build_huffman_tree(frequency_table)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Failed to build Huffman tree"))?;

    let mut codes = HashMap::new();
    generate_codes(&Some(huffman_tree), String::new(), &mut codes);

    // Build compressed bit string
    let mut compressed_bits = String::new();
    for byte in &data {
        compressed_bits.push_str(&codes[byte]);
    }

    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);

    // Create and serialize header
    let header = CompressionHeader {
        codes,
        original_bit_count: compressed_bits.len(),
    };
    let header_bytes = bincode::serialize(&header)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Serialization error: {}", e)))?;
    
    // Write header length (4 bytes) followed by header
    writer.write_all(&(header_bytes.len() as u32).to_le_bytes())?;
    writer.write_all(&header_bytes)?;

    // Convert bits to bytes and write compressed data
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

    if compressed_data.len() < 4 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Compressed file too small"));
    }

    // Read header length
    let header_len = u32::from_le_bytes([
        compressed_data[0], compressed_data[1], compressed_data[2], compressed_data[3]
    ]) as usize;

    if compressed_data.len() < 4 + header_len {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid compressed file format"));
    }

    // Deserialize header
    let header: CompressionHeader = bincode::deserialize(&compressed_data[4..4 + header_len])
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Deserialization error: {}", e)))?;

    // Create reverse lookup table for decompression
    let reverse_codes: HashMap<String, u8> = header.codes.into_iter()
        .map(|(byte, code)| (code, byte))
        .collect();

    // Convert compressed bytes back to bit string
    let compressed_bytes = &compressed_data[4 + header_len..];
    let mut all_bits = convert_bytes_to_bits(compressed_bytes);
    
    // Truncate to original bit count to remove padding
    all_bits.truncate(header.original_bit_count);

    // Decode using the Huffman codes
    let mut decompressed_data = Vec::new();
    let mut current_code = String::new();

    for bit_char in all_bits.chars() {
        current_code.push(bit_char);
        if let Some(&byte) = reverse_codes.get(&current_code) {
            decompressed_data.push(byte);
            current_code.clear();
        }
    }

    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    writer.write_all(&decompressed_data)?;

    Ok(())
}

// Convert a string of bits to bytes (properly handles binary conversion)
fn convert_bits_to_bytes(bits: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    
    // Process bits in chunks of 8, padding the last chunk if necessary
    for chunk in bits.as_bytes().chunks(8) {
        let mut byte = 0u8;
        for (i, &bit_char) in chunk.iter().enumerate() {
            if bit_char == b'1' {
                byte |= 1 << (7 - i);
            }
        }
        bytes.push(byte);
    }
    
    bytes
}

// Convert a byte slice to a string of bits
fn convert_bytes_to_bits(bytes: &[u8]) -> String {
    let mut bits = String::with_capacity(bytes.len() * 8);
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

    // Validate input file exists
    if !input_path.exists() {
        eprintln!("Error: Input file '{}' does not exist", input_path.display());
        std::process::exit(1);
    }

    match command.as_str() {
        "compress" => {
            if let Err(e) = compress_file(input_path, output_path) {
                eprintln!("Error compressing file: {}", e);
                std::process::exit(1);
            } else {
                println!("File compressed successfully to '{}'", output_path.display());
            }
        }
        "decompress" => {
            if let Err(e) = decompress_file(input_path, output_path) {
                eprintln!("Error decompressing file: {}", e);
                std::process::exit(1);
            } else {
                println!("File decompressed successfully to '{}'", output_path.display());
            }
        }
        _ => {
            eprintln!("Unknown command: '{}'. Use 'compress' or 'decompress'", command);
            std::process::exit(1);
        }
    }
}
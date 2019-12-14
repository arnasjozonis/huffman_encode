use std::fs::{File};
use std::io::prelude::*;
use std::io::{BufReader,BufWriter};
use bitbit::{BitWriter,BitReader};
use bitbit::reader::MSB;
use std::collections::HashMap;
use std::iter::Iterator;
use std::env;

const ABC_SIZE: usize = 65536;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Please provide filename argument and word length!");
        return;        
    }
    let filename = &args[1];
    let word_len_input = match (&args[2]).parse::<u32>() {
        Ok(number) => number,
        Err(e) => { 
            println!("Erorr parsing word length argument. {}. Using default value: 4", e);
            4u32
        }
    };

    println!("Compressing file {} with word length paramater: {}", filename, word_len_input);

    let (dict, unaccounted) = get_occurrencies(filename.to_string(), word_len_input);
    let root_node = generate_graph(&dict);
    let code = generate_huffman_code_tuples(root_node);
    let mut decode_dict: HashMap<(u64, u64), usize> = HashMap::new();
    let mut bits_total_from_dict: u32 = 0;
    for i in 0..dict.len() {
        if dict[i] != 0  {
            let key = code[i];
            decode_dict.insert(key, i);
            println!("Symbol: {} :::coded as::: {} (relevant bits{})", i, key.0, key.1);
            let b = (dict[i] * key.1) as u32;
            bits_total_from_dict = bits_total_from_dict + b;
        }
    }
    println!("---{}", bits_total_from_dict);
    create_compressed_file(
        code,
        decode_dict,
        word_len_input,
        filename.to_string(),
        unaccounted,
        bits_total_from_dict
    );    

}

struct Node {
    value: Option<usize>,
    w: u128,
    zero: Box<Option<Node>>,
    one: Box<Option<Node>>,
}

fn generate_graph(input_abc: &Vec<u64>) -> Node {
    let mut res = Vec::new();
    for i in 0..input_abc.len() {
        if input_abc[i] != 0 {
            res.push(Node {
                value: Some(i),
                w: input_abc[i] as u128,
                zero: Box::from(None),
                one: Box::from(None),
            })
        }
    }
    loop {
        res.sort_by(|n1, n2| n2.w.cmp(&n1.w));
        if res.len() > 1 {
            let smallest = res.pop().unwrap();
            let next_smallest = res.pop().unwrap();
            let new_node = Node {
                value: None,
                w: smallest.w + next_smallest.w,
                zero: Box::from(Some(next_smallest)),
                one: Box::from(Some(smallest)),
            };
            res.push(new_node);
        } else {
            break;
        }
    }
    res.pop().unwrap()
}

//TODO: change return type to vector with dynamic size
fn generate_huffman_code_tuples(root_node: Node) -> [(u64, u64); ABC_SIZE] {
    let mut dictionary: [(u64, u64); ABC_SIZE] = [(0,0); ABC_SIZE];

    walk_tree(root_node.zero.unwrap(), (0,0), true, &mut dictionary);
    walk_tree(root_node.one.unwrap(), (0,0), false, &mut dictionary);
    dictionary
}

fn walk_tree(node: Node, mut path: (u64, u64), zero_side: bool, dict: &mut [(u64, u64); ABC_SIZE]) {
    let (code, relevant_bits) = path;
    if zero_side {
        let new_code = code << 1;
        path = (new_code, relevant_bits + 1);
    } else {
        let new_code = code << 1 | 1;
        path = (new_code, relevant_bits + 1);
    }

    match node.value {
        Some(val) => {
            dict[val] = path.clone();
        },
        _ => {
            walk_tree(node.zero.unwrap(), path.clone(), true, dict);
            walk_tree(node.one.unwrap(), path.clone(), false, dict);
        }
    };
}

fn get_occurrencies(file_path: String, word_len: u32) ->  (Vec<u64>, Vec<u8>) {
    
    /* 
        Initialize vector (dynamic size array) of size to accomodate all possible words for given length,
        if word_len=3, then dictionary size=8, if word_len=8, then size=256.
        Dictionary index represents word, value at index - occurencies of word in file.
    */
    let abc_size = 2usize.pow(word_len);
    let mut dict = Vec::with_capacity(abc_size);
    for _ in 0..abc_size {
        dict.push(0);
    }


    let file = File::open(file_path).unwrap();
    let buf_reader = BufReader::new(file);

     /* 
        bytes buffer size represents number of how many bytes we need to read from file,
        to calculate word occurencies and don't get incomplete words left in the end.
        In other words: (bytes_buffer_size * 8)%word_len==0, for example
        if word_len = 5, then bytes_buffer_size = 5 (5*8 % 5 == 0).
    */
    let bytes_buffer_size = get_bytes_count_for_buffer(&(word_len as usize));
    let mut bits_container: u128 = 0;
    let mut bytes_buffer_cursor = bytes_buffer_size - 1;
    let mut byte_count = 0;
    let mask = 2usize.pow(word_len) - 1;
    for maybe_byte in buf_reader.bytes() {
        match maybe_byte {
            Ok(byte) => {
                bits_container = (byte as u128) << 8*bytes_buffer_cursor | bits_container;
                byte_count = byte_count + 1;
                if bytes_buffer_cursor == 0 {
                    let mut idx;
                    let words_in_container = bytes_buffer_size*8/word_len as usize;
                    
                    for i in (0..words_in_container).rev() {
                        idx = ((mask << (i*word_len as usize)) as u128 & bits_container) as usize;
                        idx = idx >> (i*word_len as usize);
                        let count = dict[idx];
                        dict[idx] = count + 1;
                    }
                    bytes_buffer_cursor = bytes_buffer_size - 1;
                    bits_container = 0;
                } else {
                    bytes_buffer_cursor = bytes_buffer_cursor - 1;
                }
            },
            _ => {
                println!("Some kind of error while reading bytes...");
                break;
            }
        }
    }
    let total_unaccounted_bytes = byte_count % bytes_buffer_size;
    let mut unaccounted_bytes: Vec<u8> = Vec::with_capacity(total_unaccounted_bytes);
    for i in (0..total_unaccounted_bytes).rev() {
        let byte = (bits_container >> 8*(i + 1)) as u8;
        unaccounted_bytes.push(byte);
    }
    println!("\nTotal {} bytes found... not compressed last {} bytes", byte_count, total_unaccounted_bytes);
    (dict, unaccounted_bytes)
}

fn get_bytes_count_for_buffer(letter_bit_count: &usize) -> usize {
    let mut result = 8usize;
    while result%letter_bit_count != 0 {
        result = result + 8usize;
    }
    result/8
}
fn write_bits(
    bw: &mut BitWriter<&mut std::io::BufWriter<std::fs::File>>,
    bits_container: u64,
    bits_count: u32,
    bit_counter: &mut u128) -> Result<(), String> 
{
    if bits_count > 64 {
        let error = String::from("bits_count argument is out of range");
        return Err(error);
    }
    for i in (0..bits_count).rev() {
        let mask = 2u64.pow(i);
        let bit = (bits_container & mask) > 0;
        bw.write_bit(bit).unwrap();
        *bit_counter = (*bit_counter) + 1;
        if *bit_counter < 96 {
            if bit {
                print!("1");
            } else {
                print!("0");
            }
        }
    }
    Ok(())
}
 
fn compare_file_bits(file_path: String, second_file: String) {
    let file = File::open(file_path.clone()).unwrap();
    let buf_reader = BufReader::new(file);
    let mut br: BitReader<_,MSB> = BitReader::new(buf_reader);

    let file2 = File::open(second_file.clone()).unwrap();
    let buf_reader2 = BufReader::new(file2);
    let mut br2: BitReader<_,MSB> = BitReader::new(buf_reader2);

    let mut i = 0;
    loop {
        let b = br.read_bit().unwrap();
        let b2 = br2.read_bit().unwrap();
        if b2 != b {
            println!("miss in byte {}", i%8);
        }
        i = i + 1;
    }
    println!("\nDone./\n");
}

fn create_compressed_file(
    code: [(u64, u64); ABC_SIZE],
    decode_dict: HashMap<(u64, u64), usize>,
    word_len: u32,
    source_filename: String,
    uncompressed_bytes: Vec<u8>,
    total_bits_dict: u32
) {
    let w = File::create(format!("{}.bdazip", source_filename)).unwrap();
    let abc_count = (decode_dict.len() as u16).to_be_bytes();
    let total_bits_from_dict = total_bits_dict.to_be_bytes();
    let mut buf_writer = BufWriter::new(w);
    let mut bw = BitWriter::new(&mut buf_writer);
    bw.write_byte(word_len as u8).unwrap();
    bw.write_byte(abc_count[0]).unwrap();
    bw.write_byte(abc_count[1]).unwrap();
    bw.write_byte(total_bits_from_dict[0]).unwrap();
    bw.write_byte(total_bits_from_dict[1]).unwrap();
    bw.write_byte(total_bits_from_dict[2]).unwrap();
    bw.write_byte(total_bits_from_dict[3]).unwrap();
    bw.write_byte(uncompressed_bytes.len() as u8).unwrap();
    let mut bit_counter = 64u128;
    let code_length_counter = 64 - (decode_dict.len() as u64).leading_zeros();
    let dict_len = decode_dict.len();
    for ((code, important_bits), symbol) in decode_dict {
        write_bits(&mut bw, important_bits.into(), code_length_counter, &mut bit_counter).unwrap();
        write_bits(&mut bw, code, important_bits as u32, &mut bit_counter).unwrap();
        write_bits(&mut bw, symbol as u64, word_len, &mut bit_counter).unwrap();
    }
    println!("\nHeader is set successfully:
        word length: {},
        dict length: {},
        dict bytes: {} ({} bits),
        uncompressed bytes: {},
        code length counter: {}\n", 
        word_len, dict_len, total_bits_dict/8, total_bits_dict, uncompressed_bytes.len(), code_length_counter);
    
    let file = File::open(source_filename).unwrap();
    let buf_reader = BufReader::new(file);
    let bytes_buffer_size = get_bytes_count_for_buffer(&(word_len as usize));
    let mut bits_container: u128 = 0;
    let mut bytes_buffer_cursor = bytes_buffer_size - 1;
    let mut byte_count = 0;
    let mask = 2usize.pow(word_len) - 1;
    for letter in buf_reader.bytes() {
        match letter {
            Ok(byte) => {
                bits_container = (byte as u128) << 8*bytes_buffer_cursor | bits_container;
                byte_count = byte_count + 1;
                if bytes_buffer_cursor == 0 {
                    let mut idx;
                    let words_in_container = bytes_buffer_size*8/word_len as usize;
                    for i in (0..words_in_container).rev() {
                        idx = ((mask << (i*word_len as usize)) as u128 & bits_container) as usize;
                        idx = idx >> (i*word_len as usize);
                        let ( code, relevant_bits) = code[idx].clone();
                        write_bits(&mut bw, code, relevant_bits as u32, &mut bit_counter).unwrap();
                    }
                    bytes_buffer_cursor = bytes_buffer_size - 1;
                    bits_container = 0;
                }
                else {
                    bytes_buffer_cursor = bytes_buffer_cursor - 1;
                }
            },
            _ => {
                println!("IO error...");
                break;
            }
        }
    }
    println!("Bit counter result before uncompressed bytes are added: {}", bit_counter);
    for byte in uncompressed_bytes {
        bw.write_byte(byte).unwrap();
        bit_counter = bit_counter + 8;
    }
    while bit_counter % 8 != 0 {
        bw.write_bit(false).unwrap();
        bit_counter += 1;
    }
    println!("Bit counter result in the end: {}", bit_counter);
    buf_writer.flush().unwrap();

}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_bytes_buffer_counter() {
        assert_eq!(get_bytes_count_for_buffer(&3), 3);
        assert_eq!(get_bytes_count_for_buffer(&8), 1);
        assert_eq!(get_bytes_count_for_buffer(&4), 1);
        assert_eq!(get_bytes_count_for_buffer(&5), 5);
        assert_eq!(get_bytes_count_for_buffer(&16), 2);
        assert_eq!(get_bytes_count_for_buffer(&13), 13);
        assert_eq!(get_bytes_count_for_buffer(&9), 9);
        assert_eq!(get_bytes_count_for_buffer(&12), 3);
    }

}
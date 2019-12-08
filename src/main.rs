use std::fs::{File};
use std::io::prelude::*;
use std::io::{BufReader,BufWriter};
use std::ops::Add;
use bitbit::{BitWriter,BitReader};
use bitbit::reader::MSB;
use std::collections::HashMap;
use std::iter::Iterator;

const ABC_SIZE: usize = 65536;

fn main() {
    println!("Encoding...");
    write_file_bits(String::from("test.txt"));
    let (dict, unaccounted) = get_occurrencies(String::from("test.txt"), 3);
    let root_node = generate_graph(&dict);
    // let total_bits_written = root_node.w.clone() * word length ??;
    let code = generate_huffman_code_tuples(root_node);
    let mut decode_dict: HashMap<(u16, u8), usize> = HashMap::new();
    println!("\n\n===== occurences =====");
    for i in 0..dict.len() {
        if dict[i] != 0  {
            println!("symbol {} found {} times", i as u8, dict[i]);
            let key = code[i];
            decode_dict.insert(key, i);
        }
    }
    for byte in unaccounted {
        println!("{}", byte as char);
    }

    let w = File::create("zipped.ajoz").unwrap();
    let abc_count = (decode_dict.len() as u16).to_be_bytes();
    
    let mut buf_writer = BufWriter::new(w);
    let mut bw = BitWriter::new(&mut buf_writer);
    bw.write_byte(abc_count[0]).unwrap();
    bw.write_byte(abc_count[1]).unwrap();

    //TODO: add leftover bytes and byte count for decoder
    //TODO: adjust bit calculations for dynamic word length
    for ((code, important_bits), symbol) in decode_dict {
        let [ c1, c2 ] = code.to_be_bytes();
        let [ s1, s2 ] = (symbol as u16).to_be_bytes();
        bw.write_byte(c1).unwrap();
        bw.write_byte(c2).unwrap();
        bw.write_byte(important_bits).unwrap();
        bw.write_byte(s1).unwrap();
        bw.write_byte(s2).unwrap();
    }
    let file = File::open("test.txt").unwrap();
    let buf_reader = BufReader::new(file);
    let mut bit_counter = 0u128;
    for letter in buf_reader.bytes() {
        match letter {
            Ok(l) => {
                let idx = l as usize;
                let coded = code[idx].clone();
                for bit in (0..coded.1).rev() {
                    let mask = 2u16.pow(bit as u32);
                    let bit = coded.0 & mask != 0;
                    bw.write_bit(bit).unwrap();
                    bit_counter = bit_counter + 1;
                    // if bit {
                    //     print!("1");
                    // } else {
                    //     print!("0");
                    // }
                }
            },
            _ => {
                println!("IO error...");
                break;
            }
        }
    }
    println!("bits written so far: {}", bit_counter);
    for i in 0..(8 - bit_counter%8) {
        println!("{}", i);
        bw.write_bit(false).unwrap();
    }
    buf_writer.flush().unwrap();
}

struct Node {
    value: Option<usize>,
    w: u128,
    zero: Box<Option<Node>>,
    one: Box<Option<Node>>,
}

fn generate_graph(input_abc: &Vec<u16>) -> Node {
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

// fn generate_huffman_code(root_node: Node) -> [String; ABC_SIZE] {
//     let mut res: [String; 256] = [
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
//         String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), 
//     ];

//     walk_tree_str(root_node.zero.unwrap(), String::from(""), true, &mut res);
//     walk_tree_str(root_node.one.unwrap(), String::from(""), false, &mut res);
//     res
// }


//TODO: change return type to vector with dynamic size
fn generate_huffman_code_tuples(root_node: Node) -> [(u16, u8); ABC_SIZE] {
    let mut res: [(u16, u8); ABC_SIZE] = [(0,0); ABC_SIZE];

    walk_tree(root_node.zero.unwrap(), (0,0), true, &mut res);
    walk_tree(root_node.one.unwrap(), (0,0), false, &mut res);
    res
}

fn walk_tree_str(node: Node, mut path: String, zero_side: bool, dict: &mut [String; ABC_SIZE]) {
    if zero_side {
        path = path.add("0");
    } else {
        path = path.add("1");
    }

    match node.value {
        Some(val) => {
            println!("final: {} {}", val as u8 as char, path);
            dict[val] = path.clone();
        },
        _ => {
            walk_tree_str(node.zero.unwrap(), path.clone(), true, dict);
            walk_tree_str(node.one.unwrap(), path.clone(), false, dict);
        }
    };
}

fn walk_tree(node: Node, mut path: (u16, u8), zero_side: bool, dict: &mut [(u16, u8); ABC_SIZE]) {
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
            println!("final: {} ({}, {})", val as u8 as char, path.0, path.1);
            dict[val] = path.clone();
        },
        _ => {
            walk_tree(node.zero.unwrap(), path.clone(), true, dict);
            walk_tree(node.one.unwrap(), path.clone(), false, dict);
        }
    };
}

fn get_occurrencies(file_path: String, word_len: u32) ->  (Vec<u16>, Vec<u8>) {
    let abc_size = 2usize.pow(word_len);
    let mut dict = Vec::with_capacity(abc_size);
    for i in 0..abc_size {
        dict.push(0);
    }
    let file = File::open(file_path).unwrap();
    let buf_reader = BufReader::new(file);
    let bytes_buffer_size = get_bytes_count_for_buffer(&(word_len as usize));
    let mut bits_container: u128 = 0;
    let mut bytes_buffer_cursor = bytes_buffer_size - 1;
    let mut byte_count = 0;
    let mask = 2usize.pow(word_len) - 1;
    println!("bytes buffer size: {}, mask: {}",bytes_buffer_size, mask);
    for maybe_byte in buf_reader.bytes() {
        match maybe_byte {
            Ok(byte) => {
                bits_container = (byte as u128) << 8*bytes_buffer_cursor | bits_container;
                byte_count = byte_count + 1;
                if bytes_buffer_cursor == 0 {
                    println!("calculating count: {}", bits_container);
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
    println!("Total {} bytes found... ", byte_count);
    (dict, unaccounted_bytes)
}

fn get_bytes_count_for_buffer(letter_bit_count: &usize) -> usize {
    let mut result = 8usize;
    while result%letter_bit_count != 0 {
        result = result + 8usize;
    }
    result/8
}

fn write_file_bits(file_path: String) {
    let file = File::open(file_path.clone()).unwrap();
    let buf_reader = BufReader::new(file);
    let mut br: BitReader<_,MSB> = BitReader::new(buf_reader);
    println!("\nWriting bits for for file {}./\n", file_path.clone());
    loop {
        let b = br.read_bit();
        match b {
            Ok(bit) => {
                if bit {
                    print!("1");
                } else {
                    print!("0");
                }
            },
            Err(err) => {
                println!("\n\n{}", err);
                break
            }
        }
    }
    println!("\nDone./\n");
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
        assert_eq!(get_bytes_count_for_buffer(&16), 2);
        assert_eq!(get_bytes_count_for_buffer(&13), 13);
        assert_eq!(get_bytes_count_for_buffer(&9), 9);
        assert_eq!(get_bytes_count_for_buffer(&12), 3);
    }

}
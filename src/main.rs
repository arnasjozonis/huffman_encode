use std::fs::{File};
use std::io::prelude::*;
use std::io::{BufReader,BufWriter};
use std::ops::Add;
use bitbit::{BitWriter,BitReader};
use bitbit::reader::MSB;
use std::collections::HashMap;
use std::iter::Iterator;

fn main() {
    println!("Encoding...");
    
    let mut dict: [u16; 256] = [0; 256];
    let file = File::open("test.txt").unwrap();
    let buf_reader = BufReader::new(file);

    for letter in buf_reader.bytes() {
        match letter {
            Ok(l) => {
                let idx = l as usize;
                let count = dict[idx];
                dict[idx] = count + 1;
            },
            _ => {
                println!("IO error...");
                break;
            }
        }
    }
    let root_node = generate_nodes(&dict);
    let total_w = root_node.w.clone();
    let code = generate_huffman_code_tuples(root_node);
    let root_node = generate_nodes(&dict);
    let code_str = generate_huffman_code(root_node);
    let mut decode_dict: HashMap<(u16, u8), usize> = HashMap::new();
    println!("\n\n===== occurences =====");
    for i in 0..dict.len() {
        if dict[i] != 0  {
            let key = code[i];
            decode_dict.insert(key, i);
            println!("Char {}: {}/{} coded as ({}, {}) <=> {}", i as u8 as char, dict[i], total_w, code[i].0, code[i].1, code_str[i]);
        }
    }

    let w = File::create("zipped.ajoz").unwrap();
    let mut buf_writer = BufWriter::new(w);
    let mut bw = BitWriter::new(&mut buf_writer);
    
    let file = File::open("test.txt").unwrap();
    let buf_reader = BufReader::new(file);
    for letter in buf_reader.bytes() {
        match letter {
            Ok(l) => {
                let idx = l as usize;
                let coded = code[idx].clone();
                for bit in (0..coded.1).rev() {
                    let mask = 2u16.pow(bit as u32);
                    let bit = coded.0 & mask != 0;
                    if bit {
                        print!("1");
                    } else {
                        print!("0"); 
                    }
                    bw.write_bit(bit).unwrap();
                }
            },
            _ => {
                println!("IO error...");
                break;
            }
        }
    }
    buf_writer.flush().unwrap();
    println!("\n");
    let file = File::open("zipped.ajoz").unwrap();
    let buff_reader = BufReader::new(file);
    let mut br: BitReader<_,MSB> = BitReader::new(buff_reader);
    let mut bits_buff = 0u16;
    let mut bit_ptr_pos = 0u8;
    loop {
        let b = br.read_bit();
        match b {
            Ok(bit) => {
                bits_buff = bits_buff << 1 | bit as u16;
                match decode_dict.get(&(bits_buff, bit_ptr_pos + 1)) {
                    Some(value) => {
                        print!("{}", *value as u8 as char);
                        bits_buff = 0;
                        bit_ptr_pos = 0;
                    },
                    None => {
                        bit_ptr_pos = bit_ptr_pos + 1;
                    }
                }
            },
            Err(err) => {
                println!("\n\n{}", err);
                break
            }
        }
    }
}

struct Node {
    value: Option<usize>,
    w: u128,
    zero: Box<Option<Node>>,
    one: Box<Option<Node>>,
}

fn generate_nodes(dict: &[u16; 256]) -> Node {
    let mut res = Vec::new();
    for i in 0..dict.len() {
        if dict[i] != 0 {
            res.push(Node {
                value: Some(i),
                w: dict[i] as u128,
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
            //println!("w1: {} w2:{}", smallest.w, next_smallest.w);
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

fn generate_huffman_code(root_node: Node) -> [String; 256] {
    let mut res: [String; 256] = [
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""),
        String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), String::from(""), 
    ];

    walk_tree_str(root_node.zero.unwrap(), String::from(""), true, &mut res);
    walk_tree_str(root_node.one.unwrap(), String::from(""), false, &mut res);
    res
}

fn generate_huffman_code_tuples(root_node: Node) -> [(u16, u8); 256] {
    let mut res: [(u16, u8); 256] = [(0,0); 256];

    walk_tree(root_node.zero.unwrap(), (0,0), true, &mut res);
    walk_tree(root_node.one.unwrap(), (0,0), false, &mut res);
    res
}

fn walk_tree_str(node: Node, mut path: String, zero_side: bool, dict: &mut [String; 256]) {
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

fn walk_tree(node: Node, mut path: (u16, u8), zero_side: bool, dict: &mut [(u16, u8); 256]) {
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
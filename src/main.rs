use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::path::Path;

fn instruction(opt: &String) -> u8 {
    match &opt[..] {
        "ADD" => 0x18,
        "AND" => 0x40,
        "COMP" => 0x28,
        "DIV" => 0x24,
        "J" => 0x3C,
        "JEQ" => 0x30,
        "JGT" => 0x34,
        "JLT" => 0x38,
        "JSUB" => 0x48,
        "LDA" => 0x00,
        "LDCH" => 0x50,
        "LDL" => 0x08,
        "LDX" => 0x04,
        "MUL" => 0x20,
        "OR" => 0x44,
        "RD" => 0xD8,
        "RSUB" => 0x4C,
        "STA" => 0x0C,
        "STCH" => 0x54,
        "STL" => 0x14,
        "STSW" => 0xE8,
        "STX" => 0x10,
        "SUB" => 0x1C,
        "TD" => 0xE0,
        "TIX" => 0x2C,
        "WD" => 0xDC,
        _ => 0xFF,
    }
}

fn is_instruction(opt: &String) -> bool {
    instruction(opt) != 0xFF
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn file_to_tokenlist(file_name: &String) -> Vec<Vec<String>> {
    let mut list = Vec::new();
    println!("[FILE] {:?}", file_name);
    if let Ok(lines) = read_lines(file_name) {
        for line in lines {
            if let Ok(s) = line {
                if s.chars().nth(0).unwrap() != '.' {
                    let mut v = Vec::new();
                    for token in s.split_whitespace() {
                        v.push(token.to_string());
                    }
                    list.push(v);
                }
            }
        }
    }
    list
}

fn has_label(line: &Vec<String>) -> bool {
    !(line.len() == 1 || is_instruction(&line[0]))
}

fn pass1(list: &Vec<Vec<String>>) -> (HashMap<String, i32>, i32) {
    let mut loc_ctr: i32 = 0;
    let mut sym_table: HashMap<String, i32> = HashMap::new();

    if list[0][0] == "START" {
        loc_ctr = i32::from_str_radix(&list[0][1][..], 16).unwrap();
    } else if list[0][1] == "START" {
        loc_ctr = i32::from_str_radix(&list[0][2][..], 16).unwrap();
    }
    let starting = loc_ctr;

    for i in 1..list.len() {
        if list[i][0] == "END" {
            break;
        }

        let mut opcode = list[i][0].clone();
        if has_label(&list[i]) {
            let label = opcode.clone();
            opcode = list[i][1].to_owned();
            if sym_table.contains_key(&opcode) {
                panic!("\n[ERROR] duplicate labels\n");
            }
            sym_table.insert(label, loc_ctr);
        }

        if is_instruction(&opcode) == true || opcode == "WORD" {
            loc_ctr += 3;
        } else if opcode == "BYTE" {
            let operand = &list[i][2];
            if operand.chars().nth(0).unwrap() == 'X' {
                loc_ctr += ((operand.len() - 3) / 2) as i32;
            } else if operand.chars().nth(0).unwrap() == 'C' {
                loc_ctr += (operand.len() - 3) as i32;
            }
        } else if opcode == "RESB" {
            loc_ctr += (list[i][2].parse::<i32>().unwrap()) as i32;
        } else if opcode == "RESW" {
            loc_ctr += (list[i][2].parse::<i32>().unwrap() * 3) as i32;
        } else {
            panic!(
                "\n[ERROR pass1] Invalid instruction/directive\n opcode {:?}\n",
                opcode
            );
        }
    }

    (sym_table, loc_ctr - starting)
}

fn file_to_obj_name(file_name: &String) -> String {
    let mut name: Vec<&str> = file_name.split('.').collect();
    name.remove(name.len() - 1);
    let name = name.pop();
    let mut name: Vec<&str> = name.unwrap().split('/').collect();
    let obj_name = name.pop();
    let mut obj_name = obj_name.unwrap().to_string();
    obj_name.push_str(".obj");
    obj_name
}

fn file_write(file: &mut File, s: &String) {
    match write!(file, "{}", s) {
        Err(why) => panic!("couldn't write to {}", why),
        Ok(_) => (),
    }
}

fn prog_name(mut s: String) -> String {
    for _ in 0..6 - s.len() {
        s.push(' ');
    }
    s
}

fn hex_str_to_word(mut s: String) -> String {
    s.make_ascii_uppercase();
    for _ in 0..6 - s.len() {
        s.insert(0, '0');
    }
    s
}

fn write_text(file: &mut File, starting: i32, line: &String) {
    let mut text = "T".to_string() + &hex_str_to_word(format!("{:x}", starting));
    let mut l = format!("{:x}", line.len() / 2);

    for _ in 0..2 - l.len() {
        l.insert(0, '0');
    }

    l.make_ascii_uppercase();
    text.push_str(&l[..].to_owned());
    text.push_str(&line[..].to_owned());
    text.push_str("\n");
    file_write(file, &text);
}

fn create_instruction(
    opcode: &String,
    operand: &mut String,
    sym_table: &HashMap<String, i32>,
) -> String {
    let mut i: i32 = instruction(opcode) as i32 * 65536;
    if operand.len() > 0 {
        if &operand[operand.len() - 2..] == ",X" {
            i += 32768;
            operand.truncate(operand.len() - 2);
        }
        if sym_table.contains_key(operand) {
            i += sym_table.get(operand).unwrap();
        } else {
            return String::from("");
        }
    }
    hex_str_to_word(String::from(format!("{:x}", i)))
}

fn pass2(
    file_name: &String,
    list: Vec<Vec<String>>,
    sym_table: HashMap<String, i32>,
    prog_len: i32,
) -> i32 {
    let mut file = match File::create(file_name) {
        Err(why) => panic!("Could not create {}", why),
        Ok(file) => file,
    };

    let mut name = String::new();
    let mut loc_ctr: i32 = 0;
    if list[0][0] == "START" {
        loc_ctr = i32::from_str_radix(&list[0][1][..], 16).unwrap();
        name = "".to_string();
    } else if list[0][1] == "START" {
        loc_ctr = i32::from_str_radix(&list[0][2][..], 16).unwrap();
        name = list[0][0].clone();
    }
    let starting: i32 = loc_ctr;

    let mut header = "H".to_string() + &prog_name(name);
    header.push_str(&hex_str_to_word(format!("{:x}", starting)));
    header.push_str(&hex_str_to_word(format!("{:x}", prog_len)));
    header.push_str("\n");
    file_write(&mut file, &header);

    let mut line = String::new();
    let mut tstart = loc_ctr;
    let mut res_flag : bool = false;

    for i in 1..list.len() {
        if list[i][0] == "END" {
            if line.len() > 0 {
                write_text(&mut file, tstart, &line);
            }
            let mut addr = starting;
            if list[i].len() == 2 {
                addr = sym_table.get(&list[i][1]).unwrap().clone();
            }

            let end = "E".to_string() + &hex_str_to_word(format!("{:x}", addr));
            file_write(&mut file, &end);
            break;
        }

        let mut opcode = list[i][0].to_owned();
        let mut operand = String::from("");
        if has_label(&list[i]) {
            // opcode is label right now, change to real opcode
            opcode = list[i][1].to_owned();
            if list[i].len() == 3 {
                operand = list[i][2].to_owned();
            }
        } else if list[i].len() == 2 {
            operand = list[i][1].to_owned();
        }

        if is_instruction(&opcode) {
            let i = create_instruction(&opcode, &mut operand, &sym_table);
            if i.len() == 0 {
                panic!("[ERROR] UNDEFINED SYMBOL");
            }
            if loc_ctr + 3 - tstart > 30 || res_flag == true {
                write_text(&mut file, tstart, &line);
                tstart = loc_ctr;
                line = i;
            } else {
                line.push_str(&i[..]);
            }
            loc_ctr += 3;
        } else if opcode == "WORD" {
            let constant = hex_str_to_word(format!("{:x}", operand.parse::<i32>().unwrap()));
            if loc_ctr + 3 - tstart > 30 {
                write_text(&mut file, tstart, &line);
                tstart = loc_ctr;
                line = constant;
            } else {
                line.push_str(&constant[..]);
            }
            loc_ctr += 3;
            res_flag = false;
        } else if opcode == "BYTE" {
            let mut constant = String::new();
            let mut operand_len: i32 = 0;
            if operand.chars().nth(0).unwrap() == 'X' {
                operand_len = ((operand.len() - 3) / 2) as i32;
                constant = operand[2..operand.len() - 1].to_string();
            } else if operand.chars().nth(0).unwrap() == 'C' {
                operand_len = (operand.len() - 3) as i32;
                for i in 2..operand.len() - 1 {
                    let mut tmp = format!("{:x}", operand.chars().nth(i).unwrap() as u8);
                    if tmp.len() == 1 {
                        tmp.insert(0, '0');
                    }
                    tmp.make_ascii_uppercase();
                    constant.push_str(&tmp[..]);
                }
            }

            if loc_ctr + operand_len - tstart > 30 {
                write_text(&mut file, tstart, &line);
                tstart = loc_ctr;
                line = constant;
            } else {
                line.push_str(&constant[..]);
            }
            loc_ctr += operand_len;
            res_flag = false;
        } else if opcode == "RESB" {
            loc_ctr += operand.parse::<i32>().unwrap();
            res_flag = true;
        } else if opcode == "RESW" {
            loc_ctr += operand.parse::<i32>().unwrap() * 3;
            res_flag = true;
        } else {
            panic!(
                "\n[ERROR pass2] Invalid instruction/directive\n opcode {:?}\n",
                opcode
            );
        }
    }

    loc_ctr - starting
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_name: &String = &args[1];

    let list = file_to_tokenlist(file_name);
    println!("token list {:?}", list);

    let (sym_table, prog_len) = pass1(&list);
    println!("[SYM TABLE] {:?}", sym_table);

    if prog_len < 0 {
        panic!("\n[ERROR] prog_len < 0\n");
    }

    let prog_len = pass2(&file_to_obj_name(file_name), list, sym_table, prog_len);
    println!("[PROG_LEN] {:?}", prog_len);
}
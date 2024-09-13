use std::fmt::format;
use std::time::SystemTime;
use crate::{get_register, get_system_register, ValueArgument, ValueTypeHeader, DATA_START};

pub fn execute(instruction_call_code: u8, value_a: ValueArgument, value_b: ValueArgument) {
    let executor: fn(ValueArgument, ValueArgument) = match instruction_call_code {
        0 => i_pass,
        1 => i_syscall,
        2 => i_move,
        3 => i_add,
        4 => i_subtract,
        5 => i_multiply,
        6 => i_divide,
        _ => {
            unknown_instruction(instruction_call_code);
            i_pass
        }
    };
    executor(value_a, value_b);
}

//FACIL INSTRUCTIONS

fn i_pass(value_a: ValueArgument, value_b: ValueArgument) {}

fn unknown_instruction(instruction_call_code: u8) {
    panic!("{}", format!("Tried to execute unknown instruction: {instruction_call_code}"));
}

//INSTRUCTIONS
fn i_syscall(value_a: ValueArgument, value_b: ValueArgument) {
    match value_a.get() as u32 {
        0 => {
            todo!()//Return the program
        },
        1 => {
            match value_b.header {
                ValueTypeHeader::LITERAL => {
                    //Print single char
                    print!("{}", value_b.get())
                }
                ValueTypeHeader::REGISTER(ref register) => {
                    let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_micros();
                    print!("VM [{timestamp}]: ");
                    let mut i = 0;

                    loop {
                        let current = get_register(&register, value_b.value + (i/2));

                        let current_byte: u8 = if (i % 2 == 0) { (current >> 8) as u8 } else { (current & 0b11111111) as u8 };

                        if (current_byte == 0) {
                            break;
                        }

                        print!("{}", current_byte as char);

                        i += 1;
                    }
                    println!();
                }
            }

        }
        _ => {}
    }
}

fn i_move(value_a: ValueArgument, value_b: ValueArgument) {
    value_b.set(value_a.get());
}

fn i_add(value_a: ValueArgument, value_b: ValueArgument) {
    value_b.set(value_a.get());
}

fn i_subtract(value_a: ValueArgument, value_b: ValueArgument) {
    value_b.set(value_a.get());
}

fn i_multiply(value_a: ValueArgument, value_b: ValueArgument) {
    value_b.set(value_a.get());
}

fn i_divide(value_a: ValueArgument, value_b: ValueArgument) {
    value_b.set(value_a.get());
}
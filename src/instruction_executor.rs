use std::fmt::format;
use crate::ValueArgument;

pub fn execute(instruction_call_code: u8, value_a: ValueArgument, value_b: ValueArgument) {
    let executor: fn(ValueArgument, ValueArgument) = match instruction_call_code {
        0b0 => i_pass,
        0b1 => i_move,
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

fn i_move(value_a: ValueArgument, value_b: ValueArgument) {
    value_b.set(value_a.get());
}
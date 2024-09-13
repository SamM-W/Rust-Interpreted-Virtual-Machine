use std::fs;
use std::str::from_utf8;

mod instruction_executor;

//3 double bytes, so total width is 6
const INSTRUCTION_WIDTH: u16 = 3;

const ACCUMULATOR: u16 = 0b00000000;
const PROGRAM_COUNTER: u16 = 0b00000001;
const STACK_START: u16 = 0b00000010;
const DATA_START: u16 = 0b00000011;

static mut VM_PROGRAM_MEMORY: [u16; 4096] = [0; 4096];
static mut VM_SYSTEM_REGISTERS: [u16; 256] = [0; 256];

const FILE_SOURCE: &str = "C:\\Onedrive Free Coding\\Java Minecraft Modding\\Forge\\VM-Draft2\\test_program.mia";

unsafe fn initialise_memory() {
    // VM_PROGRAM_MEMORY[0] = 0b0000000100000010;
    // VM_PROGRAM_MEMORY[1] = 0b0000000000000010;
    // VM_PROGRAM_MEMORY[3] = 0b0000000000000000;

    let contents = fs::read(FILE_SOURCE)
        .expect("Should have been able to read the file");

    let read_offset = 2;
    set_system_register(DATA_START, ((*contents.get_unchecked(0) as u16) << 8) + (*contents.get_unchecked(1) as u16));
    let remainder = contents.len() - read_offset;

    //Copy to program memory
    let mut i = 0;
    let mut last_written_address = 0;
    loop {
        if (remainder < i) {
            break;
        }

        let byte_left_raw: u16 = *contents.get((i) + read_offset).unwrap_or(&0) as u16;
        let byte_right_raw: u16 = *contents.get((i) + 1 + read_offset).unwrap_or(&0) as u16;

        VM_PROGRAM_MEMORY[i/2] = (byte_left_raw << 8) + byte_right_raw;
        last_written_address = i/2;
        i += 2;
    }
    set_system_register(STACK_START, (last_written_address + 1) as u16);

}

fn main() {
    unsafe { initialise_memory(); }

    println!("Program Memory dump:");
    unsafe { show_memory(&VM_PROGRAM_MEMORY); }

    execute_next();

    unsafe { println!("Final accumulator value: {}", get_system_register(ACCUMULATOR)); }
    println!("Program Memory dump:");
    unsafe { show_memory(&VM_PROGRAM_MEMORY); }
    println!("System / Kernel Memory dump:");
    unsafe { show_memory(&VM_SYSTEM_REGISTERS); }
}

fn show_memory(memory_section: &[u16]) {
    let length = memory_section.len();
    let item_per_row = 64;
    let charset = b"0123456789ABCDEF";
    let display_height = length / item_per_row;

    let mut skipped_row_count = 0;
    for i in 0..display_height {
        let mut block_text = [0; 256];
        let mut row_has_data = false;

        for j in 0..item_per_row {
            let value = memory_section[(i * item_per_row) + j];

            if value != 0 {
                row_has_data = true;
            }

            block_text[(j*4)] = charset[((value >> 12) & 0b1111) as usize];
            block_text[(j*4)+1] = charset[((value >> 8) & 0b1111) as usize];
            block_text[(j*4)+2] = charset[((value >> 4) & 0b1111) as usize];
            block_text[(j*4)+3] = charset[((value) & 0b1111) as usize];
        }

        if (row_has_data) {
            if skipped_row_count > 0 {
                println!("Empty row x{}", skipped_row_count);
                skipped_row_count = 0;
            }
            println!("{}", from_utf8(&block_text).unwrap());
        } else {
            skipped_row_count += 1;
        }
    }
}

fn execute_next() {
    let next_address = get_system_register(PROGRAM_COUNTER);
    let instruction_address = next_address * INSTRUCTION_WIDTH;
    let instruction = get_program_register(instruction_address);

    let instruction_call_code: u8 = (instruction >> 8) as u8;
    let instruction_value_headers: u8 = (instruction & 0b11111111) as u8;

    //Note that these are effectively u4, but half bytes don't exist
    let header_left = instruction_value_headers >> 4;
    let header_right = instruction_value_headers & 0b1111;

    let value_left = get_program_register(instruction_address + 1);
    let value_right = get_program_register(instruction_address + 2);

    let value_argument_left = create_value_argument(header_left, value_left);
    let value_argument_right = create_value_argument(header_right, value_right);

    instruction_executor::execute(instruction_call_code, value_argument_left, value_argument_right);

}

fn set_register(register_section: &RegisterTarget, address: u16, value: u16) {
    match register_section {
        RegisterTarget::SYSTEM => {
            set_system_register(address, value);
        }
        RegisterTarget::STACK => {
            set_program_register(address + get_system_register(DATA_START), value);
        }
        RegisterTarget::STACK => {
            set_program_register(address + get_system_register(STACK_START), value);
        }
        _ => unimplemented!()
    }
}

fn get_register(register_section: &RegisterTarget, address: u16) -> u16 {
    match register_section {
        RegisterTarget::SYSTEM => {
            get_system_register(address)
        }
        RegisterTarget::DATA => {
            get_program_register(address + get_system_register(DATA_START))
        }
        RegisterTarget::STACK => {
            get_program_register(address + get_system_register(STACK_START))
        }
        _ => unimplemented!()
    }
}

fn set_system_register(address: u16, value: u16) {
    unsafe { VM_SYSTEM_REGISTERS[address as usize] = value; }
}

fn get_system_register(address: u16) -> u16 {
    unsafe { VM_SYSTEM_REGISTERS[address as usize] }
}

fn set_program_register(address: u16, value: u16) {
    unsafe { VM_PROGRAM_MEMORY[address as usize] = value; }
}

fn get_program_register(address: u16) -> u16 {
    unsafe { VM_PROGRAM_MEMORY[address as usize] }
}

#[derive(Debug)]
pub enum RegisterTarget {
    STACK,
    DATA,
    SYSTEM,
}

pub enum ValueTypeHeader {
    LITERAL,
    REGISTER(RegisterTarget)
}

pub struct ValueArgument {
    header: ValueTypeHeader,
    value: u16
}

impl ValueArgument {
    pub fn set(&self, value: u16) {
        match &self.header {
            ValueTypeHeader::LITERAL => {
                panic!("CANNOT ASSIGN TO A WITERAL");
            }
            ValueTypeHeader::REGISTER(target) => {
                set_register(target, self.value, value);
            }
        }
    }

    pub fn get(&self) -> u16 {
        match &self.header {
            ValueTypeHeader::LITERAL => {
                self.value
            }
            ValueTypeHeader::REGISTER(target) => {
                get_register(target, self.value)
            }
        }
    }
}

fn create_value_argument(header: u8, value: u16) -> ValueArgument {
    ValueArgument {
        header: resolve_value_type_header(header),
        value
    }
}

fn resolve_value_type_header(encoded_header: u8) -> ValueTypeHeader {
    match encoded_header {
        0 => ValueTypeHeader::LITERAL,
        1 => ValueTypeHeader::REGISTER(RegisterTarget::DATA),
        2 => ValueTypeHeader::REGISTER(RegisterTarget::STACK),
        3 => ValueTypeHeader::REGISTER(RegisterTarget::SYSTEM),
        _ => panic!("{}", format!("Couldnt resolve header of bytes: {encoded_header}")),
    }
}
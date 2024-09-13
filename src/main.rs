use std::str::from_utf8;

mod instruction_executor;

const INSTRUCTION_WIDTH: u16 = 3;

const ACCUMULATOR: u16 = 0b00000000;
const PROGRAM_COUNTER: u16 = 0b00000001;
const STACK_START: u16 = 0b00000010;

static mut VM_PROGRAM_MEMORY: [u16; 4096] = [0; 4096];
static mut VM_SYSTEM_REGISTERS: [u16; 256] = [0; 256];

unsafe fn initialise_memory() {
    VM_PROGRAM_MEMORY[0] = 0b0000000100000010;
    VM_PROGRAM_MEMORY[1] = 0b0000000000000010;
    VM_PROGRAM_MEMORY[3] = 0b0000000000000000;
}

fn main() {
    unsafe { initialise_memory(); }

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
    let charset = "0123456789ABCDEF".as_bytes();
    let display_height = length / item_per_row;

    for i in 0..display_height {
        let mut block_text = [0; 256];
        for j in 0..item_per_row {
            let value = memory_section[i * item_per_row + j] as usize;

            block_text[(j*4)+1] = charset[value & 0b11111111];
            block_text[(j*4)] = charset[(value >> 8) & 0b11111111];
        }
        println!("{}", from_utf8(&block_text).unwrap());
    }
}

fn execute_next() {
    let next_address = get_system_register(PROGRAM_COUNTER);
    let instruction_address = next_address * INSTRUCTION_WIDTH;
    let instruction = get_program_register(instruction_address);

    let instruction_call_code: u8 = (instruction >> 8) as u8;
    let instruction_value_headers: u8 = (instruction & 0b11111111) as u8;

    //Note that these are effectively u4, but half bytes don't exist
    let instruction_value_left_header = instruction_value_headers >> 4;
    let instruction_value_right_header = instruction_value_headers & 0b1111;

    let value_left = get_program_register(instruction_address + 1);
    let value_right = get_program_register(instruction_address + 2);

    let value_argument_left = create_value_argument(instruction_value_left_header, value_left);
    let value_argument_right = create_value_argument(instruction_value_right_header, value_right);

    instruction_executor::execute(instruction_call_code, value_argument_left, value_argument_right);

}

fn set_register(register_section: &RegisterTarget, address: u16, value: u16) {
    match register_section {
        RegisterTarget::SYSTEM => {
            set_system_register(address, value);
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

pub enum RegisterTarget {
    STACK,
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
        0b0 => ValueTypeHeader::LITERAL,
        0b1 => ValueTypeHeader::REGISTER(RegisterTarget::STACK),
        0b10 => ValueTypeHeader::REGISTER(RegisterTarget::SYSTEM),
        _ => panic!("{}", format!("Couldnt resolve header of bytes: {encoded_header}")),
    }
}
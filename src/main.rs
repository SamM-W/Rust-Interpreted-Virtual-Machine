use log::error;

const INSTRUCTION_WIDTH: u16 = 3;

const ACCUMULATOR: u16 = 0b00000000;
const PROGRAM_COUNTER: u16 = 0b00000001;

static mut VM_PROGRAM_MEMORY: [u16; 4096] = [0; 4096];
static mut VM_SYSTEM_REGISTERS: [u16; 256] = [0; 256];

enum RegisterTarget {
    STACK,
    SYSTEM,
    PROGRAM,
}

enum ValueTypeHeader {
    LITERAL,
    REGISTER(RegisterTarget)
}

struct ValueArgument {
    header: ValueTypeHeader,
    value: u16
}

impl ValueArgument {
    fn set(&self, value: u16) {
        match &self.header {
            ValueTypeHeader::LITERAL => {
                error!("CANNOT ASSIGN TO A WITERAL");
            }
            ValueTypeHeader::REGISTER(target) => {
                set_register(target, self.value, value);
            }
        }
    }
}

fn set_register(register_section: &RegisterTarget, address: u16, value: u16) {
    match register_section {
        RegisterTarget::SYSTEM => {
            set_system_register(address, value);
        }
        _ => todo!()
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

fn main() {
    unsafe { initialise_memory(); }

    execute_next();

    unsafe { println!("Final accumulator value: {}", get_system_register(ACCUMULATOR)); }
}

fn execute_next() {
    let next_address = get_system_register(PROGRAM_COUNTER);
    let instruction_address = next_address * INSTRUCTION_WIDTH;
    let instruction = get_program_register(instruction_address);

    let instruction_call_code = instruction >> 8;
    let instruction_value_headers = instruction & 0b11111111;

    let instruction_value_left_header = instruction_value_headers >> 4;
    let instruction_value_right_header = instruction_value_headers & 0b1111;

    let value_left = get_program_register(instruction_address + 1);
    let value_right = get_program_register(instruction_address + 2);

    let value_argument_left = create_value_argument(instruction_value_left_header, value_left);
    let value_argument_right = create_value_argument(instruction_value_right_header, value_right);

}

fn create_value_argument(header: u16, value: u16) -> ValueArgument {
    ValueArgument {
        header: resolve_value_type_header(header),
        value: value
    }
}

fn resolve_value_type_header(encoded_header: u16) -> ValueTypeHeader {
    if (encoded_header & 0b1000 == 0) {
        return ValueTypeHeader::LITERAL;
    }
    match encoded_header {
        _ => ValueTypeHeader::LITERAL ,
    }
}

unsafe fn initialise_memory() {
    VM_PROGRAM_MEMORY[0] = 0b0000000100001000;
    VM_PROGRAM_MEMORY[1] = 0b0000000000000010;
    VM_PROGRAM_MEMORY[3] = 0b0000000000000000;
}

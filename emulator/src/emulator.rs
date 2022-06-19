use std::time::Instant;


enum Register {
    Arr(ArrRegister),
    Addr(AddrRegister),
}

struct ArrRegister {
    num: usize,
    name: char,
    value: u16,
}

struct AddrRegister {
    num: usize,
    name: char,
    value: u32,
}

impl std::fmt::Debug for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Arr(a) => a.name,
            Self::Addr(a) => a.name,
        };

        let value = match self {
            Self::Arr(a) => a.value as u32,
            Self::Addr(a) => a.value,
        };

        write!(f, "Reg {}: {:#010x}", name, value)
    }
}

impl Register {

    fn write(&mut self, value: u16, top: bool) {
        match self {
            Self::Arr(a) => a.value = value,
            Self::Addr(a) => {
                if top {
                    a.value = ((value as u32) << 16) | (a.value & 0xFFFF);
                } else {
                    a.value = (value as u32) | (a.value & 0xFFFF0000);
                }
            },
        }

    }

    fn read(&self, top: bool) -> u16 {
        match self {
            Self::Arr(a) => a.value,
            Self::Addr(a) => {
                if top {
                    (a.value >> 16) as u16
                } else {
                    (a.value & 0xFFFF) as u16
                }
            },
        }
    }

    fn wide_write(&mut self, value: u32) {
        match self {
            Self::Addr(a) => a.value = value,
            (_) => panic!("Provided an Arithemetic Register to wide_write function."),
        }
    }

    fn wide_read(&self) -> u32 {
        match self {
            Self::Addr(a) => a.value,
            (_) => panic!("Provided an Arithemetic Register to wide_write function."),
        }
    }
}

fn init_ArrRegister(num: usize, name: char) -> Register {
    Register::Arr(ArrRegister {
        num,
        name,
        value: 0,
    })
}

fn init_AddrRegister(num: usize, name: char) -> Register {
    Register::Addr(AddrRegister {
        num,
        name,
        value: 0,
    })
}




fn read_register(num: u16, registers: &[Register; 12]) -> u16 {
    if num < 8 {
        registers
            .get(num as usize)
            .unwrap()
            .read(false)
    } else {
        let pos = ((num / 2) + 4);
        registers
            .get(pos as usize)
            .unwrap()
            .read((num % 2) != 0)
    }
}

fn write_register(num: u16, value: u16, registers: &mut [Register; 12]){
    if num < 8 {
        registers
            .get_mut(num as usize)
            .unwrap()
            .write(value, false);
    } else {
        let pos = ((num / 2) + 4);
        registers
            .get_mut(pos as usize)
            .unwrap()
            .write(value, (num % 2) != 0);
    }
}


fn read_wide_register(num: u16, registers: &[Register; 12]) -> u32 {

    let pos = ((num / 2) + 4);
    registers
        .get(pos as usize)
        .unwrap()
        .wide_read()

}

fn write_wide_register(num: u16, value: u32, registers: &mut [Register; 12]){

    let pos = ((num / 2) + 4);
    registers
        .get_mut(pos as usize)
        .unwrap()
        .wide_write(value)

}


fn read_ram(addr: u32, ram: &Vec<u16>) -> u16 {

    if addr < 0xF000_0000 {
        match ram.get(addr as usize) {
            Some(val) => *val,
            None => 0x0000,
        }
    } else {
        panic!("Do not know how to use IO yet. (Address: {})", addr)
    }
}



fn write_ram(addr: u32, val: u16, ram: &mut Vec<u16>) {

    if addr < 0xF000_0000 {
        ram[addr as usize] = val;
    } else {
        panic!("Do not know how to use IO yet. (Address: {})", addr)
    }
}






pub fn run(mut ram: Vec<u16>) {

    let mut registers; // our 16 registers
    let mut instr: u16; // instruction "register" (only accessable by decode)
    let mut pc: u32;

    let mut a_bus: u16;
    let mut b_bus: u16;

    let mut out_bus: u16;
    let mut out_reg: u16;
    let mut out_write: bool;

    let mut ram_bus: u16;

    let mut alu_op: u16;

    let mut gt_flag: bool = false;
    let mut eq_flag: bool = false;
    let mut ls_flag: bool = false;
    let mut ov_flag: bool = false;

    let mut mem_read: bool = false;
    let mut mem_write: bool = false;



    let mut jump: bool = false;

    registers = [
        init_ArrRegister(0, 'A'),
        init_ArrRegister(1, 'B'),
        init_ArrRegister(2, 'C'),
        init_ArrRegister(3, 'D'),
        init_ArrRegister(4, 'E'),
        init_ArrRegister(5, 'F'),
        init_ArrRegister(6, 'G'),
        init_ArrRegister(7, 'H'),
        init_AddrRegister(0, 'P'),
        init_AddrRegister(1, 'J'),
        init_AddrRegister(2, 'K'),
        init_AddrRegister(3, 'L'),
    ];


    let mut loops = 0;

    loop {

        loops += 1;
        if loops == 100 {
            break;
        }

        // time measurement.
        let now = Instant::now();


//-------------------------------------- INSTRUCTION FETCH ----------------------------------------

        pc = read_wide_register(0b1000, &registers);
        println!("PC: {:#010x}", pc);

        instr = read_ram(pc, &ram);

        a_bus = 0;
        b_bus = 0;
        out_bus = 0;
        ram_bus = 0;
        alu_op = 0;

        out_reg = 0;
        out_write = false;

        mem_write = true;
        mem_read = true;

        jump = false;


        /*
        println!("Instr:  0b{:04b}_{:04b}_{:04b}_{:04b}",(instr & 0xF000) >> 12,
            (instr & 0x0F00) >> 8,(instr & 0x00F0) >> 4,(instr & 0x000F));
        */



//-------------------------------------- DECODE ---------------------------------------------------

        println!("Instr: 0b {:04b}_{:04b}_{:04b}_{:04b}",(instr & 0xF000) >> 12,
            (instr & 0x0F00) >> 8,(instr & 0x00F0) >> 4,(instr & 0x000F));


        if instr == 0 { // NOP
            break; // for now
            //continue;

        } else if instr < 0x0200 {
            panic!("Do not know what to do with {:#06x}", instr);

        } else if instr < 0x0300 { // SETFLG


            gt_flag = match (instr & 0x00C0) >> 6 {
                0b00 => false,
                0b01 => gt_flag,
                0b10 => !gt_flag,
                0b11 => true,
                (e) => {panic!("Incorrect SETFLG value {}, should be unreachable", e)},
            };

            eq_flag = match (instr & 0x0030) >> 4 {
                0b00 => false,
                0b01 => eq_flag,
                0b10 => !eq_flag,
                0b11 => true,
                (e) => {panic!("Incorrect SETFLG value {}, should be unreachable", e)},
            };

            ls_flag = match (instr & 0x000C) >> 2 {
                0b00 => false,
                0b01 => ls_flag,
                0b10 => !ls_flag,
                0b11 => true,
                (e) => {panic!("Incorrect SETFLG value {}, should be unreachable", e)},
            };

            ov_flag = match (instr & 0x0003) {
                0b00 => false,
                0b01 => ov_flag,
                0b10 => !ov_flag,
                0b11 => true,
                (e) => {panic!("Incorrect SETFLG value {}, should be unreachable", e)},
            };


            println!("Flags: {} {} {} {}", gt_flag, eq_flag, ls_flag, ov_flag);


        } else if instr < 0b0000_0011_0010_0000 { // JA


            let jump_code = (instr & 0b0000_0000_0001_1100) >> 2;
            let flags_num = (gt_flag as u16 * 4) + (eq_flag as u16 * 2) + (ls_flag as u16 * 1);

            jump = match jump_code {
                0b111 => true,
                0b000 => (jump_code % 2) == 1,
                other => jump_code == other,
            };

            if jump {
                pc = read_wide_register(((instr & 0x0003) << 1 ) | 0b1000, &registers);
            }


        } else if instr < 0b0000_0011_0100_0000 { // STORE



            write_ram(
                read_wide_register(
                    ((instr & 0x0003) << 1) | 0b1000,
                    &registers
                ),
                read_register(
                    (instr & 0x001C) >> 2,
                    &registers
                ),
                &mut ram
            );


        } else if instr < 0b0000_0011_0110_0000 { // LOAD


            write_register(
                (instr & 0x001C) >> 2,
                read_ram(
                    read_wide_register(
                        ((instr & 0x0003) << 1) | 0b1000,
                        &registers
                    ),
                    &ram
                ),
                &mut registers
            );


        } else if ((instr & 0b1110_0000_0000_0000) >> 13) == 1 { // TRA

            a_bus = read_register(instr & 0b0000_0000_0000_0111, &registers);
            b_bus = read_register((instr & 0b0000_0000_0011_1000) >> 3, &registers);


            out_reg = (instr & 0b0000_0001_1100_0000) >> 6;
            out_write = true;

            alu_op =  ((instr & 0b0001_1110_0000_0000) >> 9) | 0b10000;

            //println!("{:#06x}", a_bus);
            //println!("{:#06x}", b_bus);



        } else if instr & 0b1100_0000_0000_0000 != 0 { // IM

            a_bus = read_register(instr & 0b0000_0000_0000_0111, &registers);
            b_bus = (instr & 0b0000_1111_1100_0000) >> 6;

            //println!("{:#06x}", a_bus);
            //println!("{:#06x}", b_bus);


            out_reg = (instr & 0b0000_0000_0011_1000) >> 3;
            out_write = true;

            alu_op =  ((instr & 0b1111_0000_0000_0000) >> 12) | 0b10000;

            println!("{:#07b}", alu_op);


        } else { // Unknown
            panic!("Do not know what to do with {:#06x}", instr);
        }



        println!("Jump: {}, PC: {:#010x}", jump, pc);


//-------------------------------------- ALU ------------------------------------------------------


        if out_write {
            out_bus = match alu_op {
                0b10100 => {
                    b_bus
                },
                0b10101 => {
                    let r = a_bus.overflowing_add(b_bus);
                    ov_flag = r.1;
                    r.0
                },
                0b10110 => {
                    let r = a_bus.overflowing_sub(b_bus);
                    ov_flag = r.1;
                    r.0
                },
                0b10111 => {
                    let r = a_bus.overflowing_mul(b_bus);
                    ov_flag = r.1;
                    r.0
                },
                _ => {
                    panic!("ALU Op {:#06b} not implemented yet", alu_op);
                }
            }
        }

        //println!("Busses: {}, {}, {}", a_bus, b_bus, out_bus);


//-------------------------------------- REGISTER ACCESS ------------------------------------------

        if out_write {
            write_register(out_reg, out_bus, &mut registers);
        }

        println!("Registers: {:#?}", registers);



//-------------------------------------- INCREMENT PC ---------------------------------------------

        if !jump {
            pc += 1;
        }

        write_wide_register(0b1000, pc, &mut registers);





        // time measurement.
        let nanos = now.elapsed().as_nanos();
        let MHz = (1f64 / (nanos as f64)) * 1000.0;
        println!("{:.04} MHz", MHz);
    }

}

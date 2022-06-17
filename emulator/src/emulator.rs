use std::time::{Duration, Instant};

fn increment_AR(r: usize, mut registers: [u16; 16]) -> [u16; 16]{
    let r_0 = r - (r%2); // make r end in 0
    let r_1 = r_0 + 1;

    if r == r_0 {
        let incr = registers[r_0].overflowing_add(1);
        registers[r_0] = incr.0;
        //if we have overflowed, add one to integer above
        registers[r_1] += incr.1 as u16;

    } else {
        registers[r_1] += 1;
    }

    registers
}

fn get_wide_AdR(r: usize, registers: [u16; 16]) -> u32 {
    let r_0 = r - (r%2); // make r end in 0
    let r_1 = r_0 + 1;

    ((registers[r_1] as u32) << 16) + (registers[r_0] as u32)
}

fn put_wide_AdR(put_val: u32, r: usize, registers: &mut [u16; 16]){
    let r_0 = r - (r%2); // make r end in 0
    let r_1 = r_0 + 1;
    registers[r_0] = (put_val % (1<<16)) as u16;
    registers[r_1] = (put_val / (1<<16)) as u16;
}

pub fn run(ram: Vec<u16>) {

    let mut registers = [0u16; 16]; // our 16 registers
    let mut instr: u16; // instruction "register" (only accessable by decode)
    let mut pc: u32;

    let mut a_bus: u16;
    let mut a_reg: u16;
    let mut a_read: bool;

    let mut b_bus: u16;
    let mut b_reg: u16;
    let mut b_read: bool;

    let mut out_bus: u16;
    let mut out_reg: u16;
    let mut out_write: bool;

    let mut alu_op: u16;

    let mut gt_flag: bool = false;
    let mut eq_flag: bool = false;
    let mut ls_flag: bool = false;
    let mut ov_flag: bool = false;

    loop {
        // time measurement.
        let now = Instant::now();


//-------------------------------------- INSTRUCTION FETCH ----------------------------------------

        pc = get_wide_AdR(0b1000, registers);
        println!("PC: {:#010x}", pc);

        instr = match ram.get(pc as usize) {
            Some(val) => *val,
            None => 0x0000,
        };


        a_read = false;
        b_read = false;
        out_write = false;

        a_bus = 0;
        b_bus = 0;
        out_bus = 0;

        a_reg = 0;
        b_reg = 0;
        out_reg = 0;



        /*
        println!("Instr:  0b{:04b}_{:04b}_{:04b}_{:04b}",
            (instr & 0xF000) >> 12,
            (instr & 0x0F00) >> 8,
            (instr & 0x00F0) >> 4,
            (instr & 0x000F),
        );
        */






//-------------------------------------- DECODE ---------------------------------------------------

        if instr == 0 { // NOP
            return;

        } else if ((instr & 0b1110_0000_0000_0000) >> 13) == 1 { // TRA

            a_reg =   (instr & 0b0000_0000_0000_0111);
            a_read = true;

            b_reg =   (instr & 0b0000_0000_0011_1000) >> 3;
            b_read = true;

            out_reg = (instr & 0b0000_0001_1100_0000) >> 6;
            out_write = true;

            alu_op =  (instr & 0b0001_1110_0000_0000) >> 9;



        } else if instr & 0b1100_0000_0000_0000 != 0 { // IM

            a_reg =   (instr & 0b0000_0000_0000_0111);
            a_read = true;

            b_bus =   (instr & 0b0000_1111_1100_0000) >> 6;
            b_read = false;

            out_reg = (instr & 0b0000_0000_0011_1000) >> 3;
            out_write = true;

            alu_op =  (instr & 0b1111_0000_0000_0000) >> 12;


        } else { // NOP
            panic!("Do not know what to do with {:#06x}", instr);
        }



//-------------------------------------- REGISTER READ --------------------------------------------


        if a_read {
            a_bus = registers[a_reg as usize];
        }

        if b_read {
            b_bus = registers[b_reg as usize];
        }

        //println!("Registers: {:?}", registers);

        //println!("Busses: {}, {}, {}", a_bus, b_bus, out_bus);



//-------------------------------------- ALU ------------------------------------------------------


        if out_write {
            out_bus = match alu_op {
                0b0100 => {
                    b_bus
                },
                0b0101 => {
                    let r = a_bus.overflowing_add(b_bus);
                    ov_flag = r.1;
                    r.0
                },
                0b0110 => {
                    let r = a_bus.overflowing_sub(b_bus);
                    ov_flag = r.1;
                    r.0
                },
                0b0111 => {
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


//-------------------------------------- REGISTER WRITE -------------------------------------------



        if out_write {
            registers[out_reg as usize] = out_bus;
        }

        println!("Registers: {:?}", registers);



//-------------------------------------- INCREMENT PC ---------------------------------------------
        pc += 1;
        put_wide_AdR(pc, 0b1000, &mut registers);





        // time measurement.
        let nanos = now.elapsed().as_nanos();
        let MHz = (1f64 / (nanos as f64)) * 1000.0;
        println!("{:.04} MHz", MHz);
    }

}

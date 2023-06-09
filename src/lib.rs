enum Flag {
    C = 0,
    P = 2,
    A = 4,
    Z = 6,
    S = 7,
}

const CONSTANT_FLAGS: u8 = 0b00101010;

#[derive(Clone, Copy)]
enum Register {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Clone, Copy)]
enum RegisterPair {
    B,
    D,
    H,
    PSW,
}

pub struct I8080 {
    pc: u16,
    sp: u16,
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    flags: u8,
    cycles: usize,
    inte: bool,
    memory: Box<[u8]>,
}

impl I8080 {
    pub fn new(memory_size: usize) -> Self {
        Self {
            pc: 0,
            sp: 0,
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            flags: 0b00000010, // always: bit-1 = 1, bit-5 = 0
            cycles: 0,
            inte: false,
            memory: vec![0; memory_size].into_boxed_slice(),
        }
    }

    pub fn cycle(&mut self) {
        if self.cycles > 0 {
            self.cycles -= 1;
            return;
        }

        let opcode = self.next_u8();
        let cycles = match opcode {
            0x00 | 0x10 | 0x20 | 0x30 | 0x08 | 0x18 | 0x28 | 0x38 => 4, // NOP
            0x01 => {self.lxi(RegisterPair::B); 10},                    // LXI B,d16
            0x11 => {self.lxi(RegisterPair::D); 10},                    // LXI D,d16
            0x21 => {self.lxi(RegisterPair::H); 10},                    // LXI D,d16
            0x31 => {self.lxi_sp(); 10},                                // LXI SP,d16
 
            0xC1 => {self.pop(RegisterPair::B); 10},                    // POP B
            0xD1 => {self.pop(RegisterPair::D); 10},                    // POP D
            0xE1 => {self.pop(RegisterPair::H); 10},                    // POP H
            0xF1 => {self.pop(RegisterPair::PSW); 10},                  // POP PSW

            0xC5 => {self.push(RegisterPair::B); 11},                   // PUSH B
            0xD5 => {self.push(RegisterPair::D); 11},                   // PUSH D
            0xE5 => {self.push(RegisterPair::H); 11},                   // PUSH H
            0xF5 => {self.push(RegisterPair::PSW); 11},                 // PUSH PSW
 
            0x22 => {self.shld(); 16},                                  // SHLD a16
            0x2A => {self.lhld(); 16},                                  // LHLD a16
            0xE3 => {self.xthl(); 18},                                  // XTHL
            0xF9 => {self.sphl(); 5},                                   // SPHL
            0xEB => {self.xchg(); 5},                                   // XCHG

            0x02 => {self.stax(RegisterPair::B); 7},                    // STAX B
            0x12 => {self.stax(RegisterPair::D); 7},                    // STAX D
            0x0A => {self.ldax(RegisterPair::B); 7},                    // LDAX B
            0x1A => {self.ldax(RegisterPair::D); 7},                    // LDAX D

            0x06 => {self.mvi(Register::B); 7},                         // MVI B,d8
            0x16 => {self.mvi(Register::D); 7},                         // MVI D,d8
            0x26 => {self.mvi(Register::H); 7},                         // MVI H,d8
            0x36 => {self.mvi_m(); 10},                                 // MVI M,d8
            0x0E => {self.mvi(Register::C); 7},                         // MVI C,d8
            0x1E => {self.mvi(Register::E); 7},                         // MVI E,d8
            0x2E => {self.mvi(Register::L); 7},                         // MVI L,d8
            0x3E => {self.mvi(Register::A); 7},                         // MVI A,d8
 
            0x32 => {self.sta(); 13},                                   // STA a16
            0x3A => {self.lda(); 13},                                   // LDA a16

            0x40 => 5,                                                  // MOV B,B
            0x50 => {self.mov(Register::D, Register::B); 5},            // MOV D,B
            0x60 => {self.mov(Register::H, Register::B); 5},            // MOV H,B
            0x70 => {self.mov_m(Register::B); 7},                       // MOV M,B

            0x41 => {self.mov(Register::B, Register::C); 5},            // MOV B,C
            0x51 => {self.mov(Register::D, Register::C); 5},            // MOV D,C
            0x61 => {self.mov(Register::H, Register::C); 5},            // MOV H,C
            0x71 => {self.mov_m(Register::C); 7},                       // MOV M,C

            0x42 => {self.mov(Register::B, Register::D); 5},            // MOV B,D
            0x52 =>  5,                                                 // MOV D,D
            0x62 => {self.mov(Register::H, Register::D); 5},            // MOV H,D
            0x72 => {self.mov_m(Register::D); 7},                       // MOV M,D
            
            0x43 => {self.mov(Register::B, Register::E); 5},            // MOV B,E
            0x53 => {self.mov(Register::D, Register::E); 5},            // MOV D,E
            0x63 => {self.mov(Register::H, Register::E); 5},            // MOV H,E
            0x73 => {self.mov_m(Register::E); 7},                       // MOV M,E

            0x44 => {self.mov(Register::B, Register::H); 5},            // MOV B,H
            0x54 => {self.mov(Register::D, Register::H); 5},            // MOV H,H
            0x64 =>  5,                                                 // MOV D,H
            0x74 => {self.mov_m(Register::H); 7},                       // MOV M,H
            
            0x45 => {self.mov(Register::B, Register::L); 5},            // MOV B,L
            0x55 => {self.mov(Register::D, Register::L); 5},            // MOV D,L
            0x65 => {self.mov(Register::H, Register::L); 5},            // MOV H,L
            0x75 => {self.mov_m(Register::L); 7},                       // MOV M,L
                                                                        
            0x46 => {self.mov_m(Register::B); 7},                       // MOV B,M
            0x56 => {self.mov_m(Register::D); 7},                       // MOV D,M
            0x66 => {self.mov_m(Register::H); 7},                       // MOV H,M
            
            0x47 => {self.mov(Register::B, Register::A); 5},            // MOV B,A
            0x57 => {self.mov(Register::D, Register::A); 5},            // MOV D,A
            0x67 => {self.mov(Register::H, Register::A); 5},            // MOV H,A
            0x77 => {self.mov_m(Register::A); 7},                       // MOV M,A
            
            0x48 => {self.mov(Register::C, Register::B); 5},            // MOV C,B
            0x58 => {self.mov(Register::E, Register::B); 5},            // MOV E,B
            0x68 => {self.mov(Register::L, Register::B); 5},            // MOV L,B
            0x78 => {self.mov(Register::A, Register::B); 5},            // MOV A,B
            
            0x49 => 5,                                                  // MOV C,C
            0x59 => {self.mov(Register::E, Register::C); 5},            // MOV E,C
            0x69 => {self.mov(Register::L, Register::C); 5},            // MOV L,C
            0x79 => {self.mov(Register::A, Register::C); 5},            // MOV A,C
            
            0x4A => {self.mov(Register::C, Register::D); 5},            // MOV C,D
            0x5A => {self.mov(Register::E, Register::D); 5},            // MOV E,D
            0x6A => {self.mov(Register::L, Register::D); 5},            // MOV L,D
            0x7A => {self.mov(Register::A, Register::D); 5},            // MOV A,D
            
            0x4B => {self.mov(Register::C, Register::E); 5},            // MOV C,E
            0x5B => 5,                                                  // MOV E,E
            0x6B => {self.mov(Register::L, Register::E); 5},            // MOV L,E
            0x7B => {self.mov(Register::A, Register::E); 5},            // MOV A,E
            
            0x4C => {self.mov(Register::C, Register::H); 5},            // MOV C,H
            0x5C => {self.mov(Register::E, Register::H); 5},            // MOV E,H
            0x6C => {self.mov(Register::L, Register::H); 5},            // MOV L,H
            0x7C => {self.mov(Register::A, Register::H); 5},            // MOV A,H
            
            0x4D => {self.mov(Register::C, Register::L); 5},            // MOV C,L
            0x5D => {self.mov(Register::E, Register::L); 5},            // MOV E,L
            0x6D => 5,                                                  // MOV L,L
            0x7D => {self.mov(Register::A, Register::L); 5},            // MOV A,L
                                                                        
            0x4E => {self.mov_m(Register::C); 7},                       // MOV C,M
            0x5E => {self.mov_m(Register::E); 7},                       // MOV E,M
            0x6E => {self.mov_m(Register::L); 7},                       // MOV L,M
            0x7E => {self.mov_m(Register::A); 7},                       // MOV A,M
                                                                        
            0x4F => {self.mov(Register::C, Register::A); 5},            // MOV C,A
            0x5F => {self.mov(Register::E, Register::A); 5},            // MOV E,A
            0x6F => {self.mov(Register::L, Register::A); 5},            // MOV L,A
            0x7F => 5,                                                  // MOV A,A
 
            0x04 => {self.inr(Register::B); 5},                         // INR B
            0x14 => {self.inr(Register::D); 5},                         // INR D
            0x24 => {self.inr(Register::H); 5},                         // INR H
            0x34 => {self.inr_m(); 10},                                 // INR M
 
            0x0C => {self.inr(Register::C); 5},                         // INR C
            0x1C => {self.inr(Register::E); 5},                         // INR E
            0x2C => {self.inr(Register::L); 5},                         // INR L
            0x3C => {self.inr(Register::A); 5},                         // INR A

            0x05 => {self.dcr(Register::B); 5},                         // DCR B
            0x15 => {self.dcr(Register::D); 5},                         // DCR D
            0x25 => {self.dcr(Register::H); 5},                         // DCR H
            0x35 => {self.dcr_m(); 10},                                 // DCR M

            0x0D => {self.dcr(Register::C); 5},                         // DCR C
            0x1D => {self.dcr(Register::E); 5},                         // DCR E
            0x2D => {self.dcr(Register::L); 5},                         // DCR L
            0x3D => {self.dcr(Register::A); 5},                         // DCR A

            0x07 => {self.rlc(); 4},                                    // RLC
            0x17 => {self.ral(); 4},                                    // RAL
            0x0F => {self.rrc(); 4},                                    // RRC
            0x1F => {self.rar(); 4},                                    // RAR
            0x37 => {self.stc(); 4},                                    // STC
            0x3F => {self.cmc(); 4},                                    // CMC
            0x2F => {self.cma(); 4},                                    // CMA

            0x27 => {self.daa(); 4},                                    // DAA

            0x80 => {self.add(Register::B); 4},                         // ADD B
            0x81 => {self.add(Register::C); 4},                         // ADD C
            0x82 => {self.add(Register::D); 4},                         // ADD D
            0x83 => {self.add(Register::E); 4},                         // ADD E
            0x84 => {self.add(Register::H); 4},                         // ADD H
            0x85 => {self.add(Register::L); 4},                         // ADD L
            0x86 => {self.add_m(); 7},                                  // ADD M
            0x87 => {self.add(Register::A); 4},                         // ADD A

            0x88 => {self.adc(Register::B); 4},                         // ADC B
            0x89 => {self.adc(Register::C); 4},                         // ADC C
            0x8A => {self.adc(Register::D); 4},                         // ADC D
            0x8B => {self.adc(Register::E); 4},                         // ADC E
            0x8C => {self.adc(Register::H); 4},                         // ADC H
            0x8D => {self.adc(Register::L); 4},                         // ADC L
            0x8E => {self.adc_m(); 7},                                  // ADC M
            0x8F => {self.adc(Register::A); 4},                         // ADC A

            0x90 => {self.sub(Register::B); 4},                         // SUB B
            0x91 => {self.sub(Register::C); 4},                         // SUB C
            0x92 => {self.sub(Register::D); 4},                         // SUB D
            0x93 => {self.sub(Register::E); 4},                         // SUB E
            0x94 => {self.sub(Register::H); 4},                         // SUB H
            0x95 => {self.sub(Register::L); 4},                         // SUB L
            0x96 => {self.sub_m(); 7},                                  // SUB M
            0x97 => {self.sub(Register::A); 4},                         // SUB A

            0x98 => {self.sbb(Register::B); 4},                         // SBB B
            0x99 => {self.sbb(Register::C); 4},                         // SBB C
            0x9A => {self.sbb(Register::D); 4},                         // SBB D
            0x9B => {self.sbb(Register::E); 4},                         // SBB E
            0x9C => {self.sbb(Register::H); 4},                         // SBB H
            0x9D => {self.sbb(Register::L); 4},                         // SBB L
            0x9E => {self.sbb_m(); 7},                                  // SBB M
            0x9F => {self.sbb(Register::A); 4},                         // SBB A
 
            0xA0 => {self.ana(Register::B); 4},                         // ANA B
            0xA1 => {self.ana(Register::C); 4},                         // ANA C
            0xA2 => {self.ana(Register::D); 4},                         // ANA D
            0xA3 => {self.ana(Register::E); 4},                         // ANA E
            0xA4 => {self.ana(Register::H); 4},                         // ANA H
            0xA5 => {self.ana(Register::L); 4},                         // ANA L
            0xA6 => {self.ana_m(); 7},                                  // ANA M
            0xA7 => {self.ana(Register::A); 4},                         // ANA A

            0xA8 => {self.xra(Register::B); 4},                         // XRA B
            0xA9 => {self.xra(Register::C); 4},                         // XRA C
            0xAA => {self.xra(Register::D); 4},                         // XRA D
            0xAB => {self.xra(Register::E); 4},                         // XRA E
            0xAC => {self.xra(Register::H); 4},                         // XRA H
            0xAD => {self.xra(Register::L); 4},                         // XRA L
            0xAE => {self.xra_m(); 7},                                  // XRA M
            0xAF => {self.xra(Register::A); 4},                         // XRA A

            0xB0 => {self.ora(Register::B); 4},                         // ORA B
            0xB1 => {self.ora(Register::C); 4},                         // ORA C
            0xB2 => {self.ora(Register::D); 4},                         // ORA D
            0xB3 => {self.ora(Register::E); 4},                         // ORA E
            0xB4 => {self.ora(Register::H); 4},                         // ORA H
            0xB5 => {self.ora(Register::L); 4},                         // ORA L
            0xB6 => {self.ora_m(); 7},                                  // ORA M
            0xB7 => {self.ora(Register::A); 4},                         // ORA A

            0xB8 => {self.cmp(Register::B); 4},                         // CMP B
            0xB9 => {self.cmp(Register::C); 4},                         // CMP C
            0xBA => {self.cmp(Register::D); 4},                         // CMP D
            0xBB => {self.cmp(Register::E); 4},                         // CMP E
            0xBC => {self.cmp(Register::H); 4},                         // CMP H
            0xBD => {self.cmp(Register::L); 4},                         // CMP L
            0xBE => {self.cmp_m(); 7},                                  // CMP M
            0xBF => {self.cmp(Register::A); 4},                         // CMP A

            0xC6 => {self.adi(); 7},                                    // ADI d8
            0xCE => {self.aci(); 7},                                    // ACI d8
            0xD6 => {self.sui(); 7},                                    // SUI d8
            0xDE => {self.sbi(); 7},                                    // SBI d8
            0xE6 => {self.ani(); 7},                                    // ANI d8
            0xEE => {self.xri(); 7},                                    // XRI d8
            0xF6 => {self.ori(); 7},                                    // ORI d8
            0xFE => {self.cpi(); 7},                                    // CPI d8

            0x03 => {self.inx(RegisterPair::B); 5},                     // INX B
            0x13 => {self.inx(RegisterPair::D); 5},                     // INX D
            0x23 => {self.inx(RegisterPair::H); 5},                     // INX H
            0x33 => {self.inx_sp(); 5},                                 // INX SP

            0x0B => {self.dcx(RegisterPair::B); 5},                     // DCX B
            0x1B => {self.dcx(RegisterPair::D); 5},                     // DCX D
            0x2B => {self.dcx(RegisterPair::H); 5},                     // DCX H
            0x3B => {self.dcx_sp(); 5},                                 // DCX SP

            0x09 => {self.dad(RegisterPair::B); 10},                    // DAD B
            0x19 => {self.dad(RegisterPair::D); 10},                    // DAD D
            0x29 => {self.dad(RegisterPair::H); 10},                    // DAD H
            0x39 => {self.dad_sp(); 10},                                // DAD SP

            0xC9 | 0xD9 => {self.ret(); 10},                            // RET
            0xD8 => {self.rc(); 5},                                     // RC
            0xD0 => {self.rnc(); 5},                                    // RNC
            0xC8 => {self.rz(); 5},                                     // RZ
            0xC0 => {self.rnz(); 5},                                    // RNZ
            0xF8 => {self.rm(); 5},                                     // RM
            0xF0 => {self.rp(); 5},                                     // RP
            0xE8 => {self.rpe(); 5},                                    // RPE
            0xE0 => {self.rpo(); 5},                                    // RPO

            0xE9 => {self.pchl(); 5},                                   // PCHL
            0xC3 | 0xCB => {self.jmp(); 10},                            // JMP a16
            0xDA => {self.jc(); 10},                                    // JC a16
            0xD2 => {self.jnc(); 10},                                   // JNC a16
            0xCA => {self.jz(); 10},                                    // JZ a16
            0xC2 => {self.jnz(); 10},                                   // JNZ a16
            0xFA => {self.jm(); 10},                                    // JM a16
            0xF2 => {self.jp(); 10},                                    // JP a16
            0xEA => {self.jpe(); 10},                                   // JPE a16
            0xE2 => {self.jpo(); 10},                                   // JPO a16

            0xCD | 0xDD | 0xED | 0xFD  => {self.call(); 17},            // CALL a16
            0xDC => {self.cc(); 11},                                    // CC a16
            0xD4 => {self.cnc(); 11},                                   // CNC a16
            0xCC => {self.cz(); 11},                                    // CZ a16
            0xC4 => {self.cnz(); 11},                                   // CNZ a16
            0xFC => {self.cm(); 11},                                    // CM a16
            0xF4 => {self.cp(); 11},                                    // CP a16
            0xEC => {self.cpe(); 11},                                   // CPE a16
            0xE4 => {self.cpo(); 11},                                   // CPO a16

            0xC7 => {self.rst(0); 11},                                  // RST 0
            0xD7 => {self.rst(2); 11},                                  // RST 2
            0xE7 => {self.rst(4); 11},                                  // RST 4
            0xF7 => {self.rst(6); 11},                                  // RST 6
                                                                        //
            0xCF => {self.rst(1); 11},                                  // RST 1
            0xDF => {self.rst(3); 11},                                  // RST 3
            0xEF => {self.rst(5); 11},                                  // RST 5
            0xFF => {self.rst(7); 11},                                  // RST 7

            0xFB => {self.inte = true; 4},                              // EI
            0xF3 => {self.inte = false; 4},                             // DI

            _ => {eprintln!("Invalid opcode: {opcode}"); 0}
        };

        self.cycles += cycles;
    }

    fn read_u8(&self, location: u16) -> u8 {
        self.memory[location as usize]
    }

    fn read_u16(&self, location: u16) -> u16 {
        ((self.memory[(location + 1) as usize] as u16) << 8) | self.memory[location as usize] as u16
    }

    fn read_m(&self) -> u8 {
        self.memory[self.get_register_pair(RegisterPair::H) as usize]
    }

    fn write_u8(&mut self, location: u16, value: u8) {
        self.memory[location as usize] = value;
    }

    fn write_u16(&mut self, location: u16, value: u16) {
        let value = value.to_le_bytes();
        self.memory[location as usize] = value[0];
        self.memory[(location + 1) as usize] = value[1];
    }

    fn write_m(&mut self, value: u8) {
        self.memory[self.get_register_pair(RegisterPair::H) as usize] = value;
    }

    fn next_u8(&mut self) -> u8 {
        let value = self.read_u8(self.pc);
        self.pc += 1;
        value
    }

    fn next_u16(&mut self) -> u16 {
        let value = self.read_u16(self.pc);
        self.pc += 2;
        value
    }

    fn register_to_ref(&self, register: Register) -> &u8 {
        match register {
            Register::A => &self.a,
            Register::F => &self.flags,
            Register::B => &self.b,
            Register::C => &self.c,
            Register::D => &self.d,
            Register::E => &self.e,
            Register::H => &self.h,
            Register::L => &self.l,
        }
    }

    fn register_to_mut_ref(&mut self, register: Register) -> &mut u8 {
        match register {
            Register::A => &mut self.a,
            Register::F => &mut self.flags,
            Register::B => &mut self.b,
            Register::C => &mut self.c,
            Register::D => &mut self.d,
            Register::E => &mut self.e,
            Register::H => &mut self.h,
            Register::L => &mut self.l,
        }
    }

    fn set_register(&mut self, register: Register, value: u8) {
        *self.register_to_mut_ref(register) = value;
    }
 
    fn get_register(&mut self, register: Register) -> u8 {
        *self.register_to_ref(register)
    }
 
    fn register_pair_to_refs(&self, pair: RegisterPair) -> (&u8, &u8) {
        match pair {
            RegisterPair::B => (&self.b, &self.c),
            RegisterPair::D => (&self.d, &self.e),
            RegisterPair::H => (&self.h, &self.l),
            RegisterPair::PSW => (&self.a, &self.flags),
        }
    }
    
    fn register_pair_to_mut_refs(&mut self, pair: RegisterPair) -> (&mut u8, &mut u8) {
        match pair {
            RegisterPair::B => (&mut self.b, &mut self.c),
            RegisterPair::D => (&mut self.d, &mut self.e),
            RegisterPair::H => (&mut self.h, &mut self.l),
            RegisterPair::PSW => (&mut self.a, &mut self.flags),
        }
    }

    fn set_register_pair(&mut self, pair: RegisterPair, value: u16) {
        let value = value.to_le_bytes();
        let (high, low) = self.register_pair_to_mut_refs(pair);
        *low = value[0];
        *high = value[1];
    }

    fn get_register_pair(&self, pair: RegisterPair) -> u16 {
        let (high, low) = self.register_pair_to_refs(pair);
        ((*high as u16) << 8) | (*low as u16)
    }

    fn get_carry(&self) -> u8 {
        self.flags & 0x1
    }

    fn set_carry(&mut self, value: u8) {
        self.flags = (self.flags & !1) | value;
    }

    fn get_flag(&self, flag: Flag) -> bool {
        ((self.flags >> (flag as u8)) & 0x1) != 0
    }
 
    fn parity(value: u8) -> bool {
        let mut value = value;
        let mut sum = 0;
        while value > 0 {
            sum += value & 0x1;
            value >>= 1;
        }
        sum % 2 == 0
    }
    #[allow(arithmetic_overflow)]
    fn set_flags(&mut self, value: u16) {
        let flags = ((Self::parity((value & 0xFF) as u8) as u8) << Flag::P as u8) |
                    (((value > 0xF) as u8) << Flag::A as u8) |
                    (((value == 0) as u8) << Flag::Z as u8) |
                    ((((value & 0x80) > 0) as u8) << Flag::S as u8) |
                    (((value > 0xFF) as u8) << Flag::C as u8);
        let mask = !CONSTANT_FLAGS;
        self.flags = (self.flags & !mask) | (flags & mask);
    }
    #[allow(arithmetic_overflow)]
    fn set_flags_without_carry(&mut self, value: u8) {
        let flags = ((Self::parity(value) as u8) << Flag::P as u8) |
                    (((value > 0xF) as u8) << Flag::A as u8) |
                    (((value == 0) as u8) << Flag::Z as u8) |
                    ((((value & 0x80) > 0) as u8) << Flag::S as u8);
        let mask = !(CONSTANT_FLAGS | (1 << Flag::C as u8));
        self.flags = (self.flags & !mask) | (flags & mask);
    }

    fn lxi(&mut self, pair: RegisterPair) {
        let value = self.next_u16();
        self.set_register_pair(pair, value);
    }
    
    fn lxi_sp(&mut self) {
        self.sp = self.next_u16();
    }

    fn pop(&mut self, pair: RegisterPair) {
        let value = self.read_u16(self.sp);
        self.set_register_pair(pair, value);
        self.sp += 2;
    }

    fn push(&mut self, pair: RegisterPair) {
        let value = self.get_register_pair(pair);
        self.sp -= 2;
        self.write_u16(self.sp, value);
    }

    fn shld(&mut self) {
        let location = self.next_u16();
        let value = self.get_register_pair(RegisterPair::H);
        self.write_u16(location, value);
    }

    fn lhld(&mut self) {
        let location = self.next_u16();
        let value = self.read_u16(location);
        self.set_register_pair(RegisterPair::H, value);
    }

    fn xthl(&mut self) {
        let register = self.get_register_pair(RegisterPair::H);
        let stack = self.read_u16(self.sp);
        self.set_register_pair(RegisterPair::H, stack);
        self.write_u16(self.sp, register);
    }

    fn sphl(&mut self) {
        self.sp = self.get_register_pair(RegisterPair::H);
    }

    fn xchg(&mut self) {
        let hl = self.get_register_pair(RegisterPair::H);
        let de = self.get_register_pair(RegisterPair::D);
        self.set_register_pair(RegisterPair::H, de);
        self.set_register_pair(RegisterPair::D, hl);
    }

    fn stax(&mut self, pair: RegisterPair) {
        let location = self.get_register_pair(pair);
        self.write_u8(location, self.a);
    }
    
    fn ldax(&mut self, pair: RegisterPair) {
        let location = self.get_register_pair(pair);
        self.a = self.read_u8(location);
    }

    fn mvi(&mut self, register: Register) {
        let value = self.next_u8();
        self.set_register(register, value);
    }
 
    fn mvi_m(&mut self) {
        let value = self.next_u8();
        self.write_m(value);
    }

    fn sta(&mut self) {
        let location = self.get_register_pair(RegisterPair::H);
        self.write_u8(location, self.a);
    }

    fn lda(&mut self) {
        let location = self.get_register_pair(RegisterPair::H);
        self.a = self.read_u8(location);
    }

    fn mov(&mut self, dst: Register, src: Register) {
        let value = self.get_register(src);
        self.set_register(dst, value);
    }

    fn mov_m(&mut self, register: Register) {
        let value = self.get_register(register);
        self.write_m(value);
    }

    fn inr(&mut self, register: Register) {
        let value = self.get_register(register) + 1;
        self.set_flags_without_carry(value);
        self.set_register(register, value);
    }
 
    fn inr_m(&mut self) {
        let value = self.read_m() + 1;
        self.set_flags_without_carry(value);
        self.write_m(value);
    }
 
    fn dcr(&mut self, register: Register) {
        let value = self.get_register(register) - 1;
        self.set_flags_without_carry(value);
        self.set_register(register, value);
    }
 
    fn dcr_m(&mut self) {
        let value = self.read_m() - 1;
        self.set_flags_without_carry(value);
        self.write_m(value);
    }

    fn rlc(&mut self) {
        let carry = (self.a & 0x80) >> 7;
        self.a <<= 1;
        self.a = (!1 & self.a) | carry;
        self.set_carry(carry);
    }

    fn ral(&mut self) {
        let carry = (self.a & 0x80) >> 7;
        self.a <<= 1;
        self.a = (!1 & self.a) | self.get_carry();
        self.set_carry(carry);
    }

    fn rrc(&mut self) {
        let carry = self.a & 0x1;
        self.a >>= 1;
        self.a = (!0x80 & self.a) | (carry << 7);
        self.set_carry(carry);
    }

    fn rar(&mut self) {
        let carry = self.a & 0x1;
        self.a >>= 1;
        self.a = (!0x80 & self.a) | (self.get_carry() << 7);
        self.set_carry(carry);
    }

    fn stc(&mut self) {
        self.set_carry(1);
    }

    fn cmc(&mut self) {
        self.set_carry(!self.get_carry())
    }

    fn cma(&mut self) {
        self.a = !self.a
    }

    fn daa(&mut self) {
        let low = (self.a as u16) & 0xF;
        let mut a = self.a as u16;
        if low > 9 || self.get_flag(Flag::A) {
            a += 6;
            self.set_flags(a);
            self.a = (a & 0xFF) as u8;
        }
        let high = ((self.a as u16) & 0xF0) >> 4;
        a = self.a as u16;
        if high > 9 || self.get_flag(Flag::C) {
            a += 0x60;
            self.set_flags(a);
            self.a = (a & 0xFF) as u8;
        }
    }

    fn add(&mut self, register: Register) {
        let a = (self.a as u16) + (self.get_register(register) as u16);
        self.set_flags(a);
        self.a = (a & 0xFF) as u8;
    }
 
    fn add_m(&mut self) {
        let a = (self.a as u16) + (self.read_m() as u16);
        self.set_flags(a);
        self.a = (a & 0xFF) as u8;
    }

    fn adc(&mut self, register: Register) {
        let a = (self.a as u16) + (self.get_register(register) as u16) + (self.get_carry() as u16);
        self.set_flags(a);
        self.a = (a & 0xFF) as u8;
    }
 
    fn adc_m(&mut self) {
        let a = (self.a as u16) + (self.read_m() as u16) + (self.get_carry() as u16);
        self.set_flags(a);
        self.a = (a & 0xFF) as u8;
    }
 
    fn sub(&mut self, register: Register) {
        let a = (self.a as u16) - (self.get_register(register) as u16);
        self.set_flags(a);
        self.a = (a & 0xFF) as u8;
    }
 
    fn sub_m(&mut self) {
        let a = (self.a as u16) - (self.read_m() as u16);
        self.set_flags(a);
        self.a = (a & 0xFF) as u8;
    }

    fn sbb(&mut self, register: Register) {
        let a = (self.a as u16) - (self.get_register(register) as u16) - (self.get_carry() as u16);
        self.set_flags(a);
        self.a = (a & 0xFF) as u8;
    }
 
    fn sbb_m(&mut self) {
        let a = (self.a as u16) - (self.read_m() as u16) - (self.get_carry() as u16);
        self.set_flags(a);
        self.a = (a & 0xFF) as u8;
    }

    fn ana(&mut self, register: Register) {
        self.a &= self.get_register(register);
        self.set_flags(self.a as u16);
    }
 
    fn ana_m(&mut self) {
        self.a &= self.read_m();
        self.set_flags(self.a as u16);
    }
 
    fn xra(&mut self, register: Register) {
        self.a ^= self.get_register(register);
        self.set_flags(self.a as u16);
    }
 
    fn xra_m(&mut self) {
        self.a ^= self.read_m();
        self.set_flags(self.a as u16);
    }
    
    fn ora(&mut self, register: Register) {
        self.a |= self.get_register(register);
        self.set_flags(self.a as u16);
    }
 
    fn ora_m(&mut self) {
        self.a |= self.read_m();
        self.set_flags(self.a as u16);
    }

    fn cmp(&mut self, register: Register) {
        let a = (self.a as u16) - (self.get_register(register) as u16);
        self.set_flags(a);
    }
 
    fn cmp_m(&mut self) {
        let a = (self.a as u16) - (self.read_m() as u16);
        self.set_flags(a);
    }

    fn adi(&mut self) {
        let a = (self.a as u16) + (self.next_u8() as u16);
        self.set_flags(a);
        self.a = (a & 0xFF) as u8;
    }
 
    fn aci(&mut self) {
        let a = (self.a as u16) + (self.next_u8() as u16) + (self.get_carry() as u16);
        self.set_flags(a);
        self.a = (a & 0xFF) as u8;
    }

    fn sui(&mut self) {
        let a = (self.a as u16) - (self.next_u8() as u16);
        self.set_flags(a);
        self.a = (a & 0xFF) as u8;
    }

    fn sbi(&mut self) {
        let a = (self.a as u16) - (self.next_u8() as u16) - (self.get_carry() as u16);
        self.set_flags(a);
        self.a = (a & 0xFF) as u8;
    }

    fn ani(&mut self) {
        self.a &= self.next_u8();
        self.set_flags(self.a as u16);
    }

    fn xri(&mut self) {
        self.a ^= self.next_u8();
        self.set_flags(self.a as u16);
    }
 
    fn ori(&mut self) {
        self.a |= self.next_u8();
        self.set_flags(self.a as u16);
    }
 
    fn cpi(&mut self) {
        let a = (self.a as u16) - (self.next_u8() as u16);
        self.set_flags(a);
    }

    fn inx(&mut self, pair: RegisterPair) {
        self.set_register_pair(pair, self.get_register_pair(pair) + 1);
    }

    fn inx_sp(&mut self) {
        self.sp += 1;
    }

    fn dcx(&mut self, pair: RegisterPair) {
        self.set_register_pair(pair, self.get_register_pair(pair) - 1);
    }

    fn dcx_sp(&mut self) {
        self.sp -= 1;
    }

    fn dad(&mut self, pair: RegisterPair) {
        let sum = (self.get_register_pair(pair) as u32) + (self.get_register_pair(RegisterPair::H) as u32);
        self.set_carry((sum > 0xFFFF) as u8);
        self.set_register_pair(RegisterPair::H, (sum & 0xFFFF) as u16);
    }
 
    fn dad_sp(&mut self) {
        let sum = (self.sp as u32) + (self.get_register_pair(RegisterPair::H) as u32);
        self.set_carry((sum > 0xFFFF) as u8);
        self.set_register_pair(RegisterPair::H, (sum & 0xFFFF) as u16);
    }

    fn ret(&mut self) {
        self.pc = self.read_u16(self.sp);
        self.sp += 2;
    }

    fn rc(&mut self) {
        if self.get_flag(Flag::C) {
            self.ret();
            self.cycles += 6;
        }
    }

    fn rnc(&mut self) {
        if !self.get_flag(Flag::C) {
            self.ret();
            self.cycles += 6;
        }
    }
 
    fn rz(&mut self) {
        if self.get_flag(Flag::Z) {
            self.ret();
            self.cycles += 6;
        }
    }
 
    fn rnz(&mut self) {
        if !self.get_flag(Flag::Z) {
            self.ret();
            self.cycles += 6;
        }
    }
 
    fn rm(&mut self) {
        if self.get_flag(Flag::S) {
            self.ret();
            self.cycles += 6;
        }
    }
 
    fn rp(&mut self) {
        if !self.get_flag(Flag::S) {
            self.ret();
            self.cycles += 6;
        }
    }

    fn rpe(&mut self) {
        if self.get_flag(Flag::P) {
            self.ret();
            self.cycles += 6;
        }
    }

    fn rpo(&mut self) {
        if !self.get_flag(Flag::P) {
            self.ret();
            self.cycles += 6;
        }
    }

    fn pchl(&mut self) {
        self.pc = self.get_register_pair(RegisterPair::H);
    }

    fn jmp(&mut self) {
        self.pc = self.next_u16();
    }
 
    fn jc(&mut self) {
        if self.get_flag(Flag::C) {
            self.jmp();
        } else {
            self.pc += 2;
        }
    }
 
    fn jnc(&mut self) {
        if !self.get_flag(Flag::C) {
            self.jmp();
        } else {
            self.pc += 2;
        }
    }
 
    fn jz(&mut self) {
        if self.get_flag(Flag::Z) {
            self.jmp();
        } else {
            self.pc += 2;
        }
    }
 
    fn jnz(&mut self) {
        if !self.get_flag(Flag::Z) {
            self.jmp();
        } else {
            self.pc += 2;
        }
    }
 
    fn jm(&mut self) {
        if self.get_flag(Flag::S) {
            self.jmp();
        } else {
            self.pc += 2;
        }
    }
 
    fn jp(&mut self) {
        if !self.get_flag(Flag::S) {
            self.jmp();
        } else {
            self.pc += 2;
        }
    }

    fn jpe(&mut self) {
        if self.get_flag(Flag::P) {
            self.jmp();
        } else {
            self.pc += 2;
        }
    }

    fn jpo(&mut self) {
        if !self.get_flag(Flag::P) {
            self.jmp();
        } else {
            self.pc += 2;
        }
    }

    fn call(&mut self) {
        self.sp -= 2;
        self.write_u16(self.sp, self.pc + 2);
        self.jmp();
    }

    fn cc(&mut self) {
        if self.get_flag(Flag::C) {
            self.call();
            self.cycles += 6;
        } else {
            self.pc += 2;
        }
    }

    fn cnc(&mut self) {
        if !self.get_flag(Flag::C) {
            self.call();
            self.cycles += 6;
        } else {
            self.pc += 2;
        }
    }
 
    fn cz(&mut self) {
        if self.get_flag(Flag::Z) {
            self.call();
            self.cycles += 6;
        } else {
            self.pc += 2;
        }
    }

    fn cnz(&mut self) {
        if !self.get_flag(Flag::Z) {
            self.call();
            self.cycles += 6;
        } else {
            self.pc += 2;
        }
    }

    fn cm(&mut self) {
        if self.get_flag(Flag::S) {
            self.call();
            self.cycles += 6;
        } else {
            self.pc += 2;
        }
    }

    fn cp(&mut self) {
        if !self.get_flag(Flag::S) {
            self.call();
            self.cycles += 6;
        } else {
            self.pc += 2;
        }
    }

    fn cpe(&mut self) {
        if self.get_flag(Flag::P) {
            self.call();
            self.cycles += 6;
        } else {
            self.pc += 2;
        }
    }

    fn cpo(&mut self) {
        if !self.get_flag(Flag::P) {
            self.call();
            self.cycles += 6;
        } else {
            self.pc += 2;
        }
    }

    fn rst(&mut self, value: u8) {
        self.sp -= 2;
        self.write_u16(self.sp, self.pc);
        self.pc = (value << 3) as u16;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TESTS_DEFAULT_MEMORY_SIZE: usize = 1024;
    const TESTS_DEFAULT_SP: u16 = TESTS_DEFAULT_MEMORY_SIZE as u16 / 2;
    macro_rules! i8080 {
        () => {
            {
                let mut i8080 = I8080::new(TESTS_DEFAULT_MEMORY_SIZE);
                i8080.sp = TESTS_DEFAULT_SP;
                i8080
            }
        };
        ( $( $x:expr ), * ) => {
            {
                let mut i8080 = I8080::new(TESTS_DEFAULT_MEMORY_SIZE);
                i8080.sp = TESTS_DEFAULT_SP;
                let mut index = 0;
                $(
                    #[allow(unused_assignments)]
                    {
                        i8080.memory[index] = $x;
                        index += 1;
                    }
                 )*
                i8080
            }
        };
    }

    #[cfg(test)]
    mod util_tests {
        use super::*;
        #[test]
        fn set_register_pair() {
            let mut i8080 = i8080!();
            i8080.set_register_pair(RegisterPair::H, 0x1234);
            assert_eq!(i8080.h, 0x12);
            assert_eq!(i8080.l, 0x34);
        }
        #[test]
        fn parity() {
            assert_eq!(I8080::parity(26), false);
            assert_eq!(I8080::parity(10), true);
        }
    }

    #[cfg(test)]
    mod opcode_tests {
        use super::*;
        #[test]
        fn lxi() {
            let mut i8080 = i8080![0x3, 0x1];
            i8080.lxi(RegisterPair::H);
            assert_eq!(i8080.h, 0x1);
            assert_eq!(i8080.l, 0x3);
        }
        #[test]
        fn lxi_sp() {
            let mut i8080 = i8080![0xBC, 0x3A];
            i8080.lxi_sp();
            assert_eq!(i8080.sp, 0x3ABC);
        }
        #[test]
        fn pop() {
            let mut i8080 = i8080!();
            i8080.write_u8(i8080.sp, 0x3D);
            i8080.write_u8(i8080.sp + 1, 0x93);
            i8080.pop(RegisterPair::H);
            assert_eq!(i8080.h, 0x93);
            assert_eq!(i8080.l, 0x3D);
            assert_eq!(i8080.sp, TESTS_DEFAULT_SP + 2);
        }

        #[test]
        fn push() {
            let mut i8080 = i8080!();
            i8080.d = 0x8F;
            i8080.e = 0x9D;
            i8080.push(RegisterPair::D);
            assert_eq!(i8080.d, 0x8F);
            assert_eq!(i8080.e, 0x9D);
            assert_eq!(i8080.sp, TESTS_DEFAULT_SP - 2);
            assert_eq!(i8080.read_u8(i8080.sp), 0x9D);
            assert_eq!(i8080.read_u8(i8080.sp + 1), 0x8F);
        }
        #[test]
        fn shld() {
            let mut i8080 = i8080![0xA, 0x1];
            i8080.h = 0xAE;
            i8080.l = 0x29;
            i8080.shld();
            assert_eq!(i8080.read_u8(0x10A), 0x29);
            assert_eq!(i8080.read_u8(0x10B), 0xAE);
        }
        #[test]
        fn lhld() {
            let mut i8080 = i8080![0x5B, 0x2];
            i8080.write_u8(0x25B, 0xFF);
            i8080.write_u8(0x25C, 0x03);
            i8080.lhld();
            assert_eq!(i8080.l, 0xFF);
            assert_eq!(i8080.h, 0x03);
        }
        #[test]
        fn xthl() {
            let mut i8080 = i8080!();
            i8080.write_u8(i8080.sp, 0xF0);
            i8080.write_u8(i8080.sp + 1, 0x0D);
            i8080.h = 0x0B;
            i8080.l = 0x3C;
            i8080.xthl();
            assert_eq!(i8080.read_u8(i8080.sp), 0x3C);
            assert_eq!(i8080.read_u8(i8080.sp + 1), 0x0B);
            assert_eq!(i8080.h, 0x0D);
            assert_eq!(i8080.l, 0xF0);
        }
        #[test]
        fn sphl() {
            let mut i8080 = i8080!();
            i8080.h = 0x50;
            i8080.l = 0x6C;
            i8080.sphl();
            assert_eq!(i8080.sp, 0x506C);
        }
        #[test]
        fn xchg() {
            let mut i8080 = i8080!();
            i8080.d = 0x33;
            i8080.e = 0x55;
            i8080.h = 0x00;
            i8080.l = 0xFF;
            i8080.xchg();
            assert_eq!(i8080.d, 0x00);
            assert_eq!(i8080.e, 0xFF);
            assert_eq!(i8080.h, 0x33);
            assert_eq!(i8080.l, 0x55);
        }
        #[test]
        fn stax() {
            let mut i8080 = i8080!();
            i8080.a = 0xCC;
            i8080.b = 0x02;
            i8080.c = 0x16;
            i8080.stax(RegisterPair::B);
            assert_eq!(i8080.read_u8(0x0216), i8080.a);
        }
        #[test]
        fn ldax() {
            let mut i8080 = i8080!();
            i8080.write_u8(0x0216, 0xCC);
            i8080.d = 0x02;
            i8080.e = 0x16;
            i8080.ldax(RegisterPair::D);
            assert_eq!(i8080.read_u8(0x0216), i8080.a);
        }
        #[test]
        fn mvi() {
            let mut i8080 = i8080![0x02, 0x34, 0xCC];
            i8080.mvi(Register::H);
            assert_eq!(i8080.get_register(Register::H), 0x02);
            i8080.mvi(Register::L);
            assert_eq!(i8080.get_register(Register::L), 0x34);
            i8080.mvi_m();
            assert_eq!(i8080.read_u8(0x0234), 0xCC);
        }
        #[test]
        fn mvi_m() {
            let mut i8080 = i8080![0xCC];
            let location = 0x0234;
            i8080.set_register_pair(RegisterPair::H, location);
            i8080.mvi_m();
            assert_eq!(i8080.read_u8(location), 0xCC);
        }
        #[test]
        fn sta() {
            let mut i8080 = i8080!();
            i8080.set_register_pair(RegisterPair::H, 0x0234);
            i8080.set_register(Register::A, 0x12);
            i8080.sta();
            assert_eq!(i8080.get_register(Register::A), i8080.read_u8(0x0234));
        }
        #[test]
        fn lda() {
            let mut i8080 = i8080!();
            i8080.write_u8(0x300, 0xFE);
            i8080.set_register_pair(RegisterPair::H, 0x300);
            i8080.lda();
            assert_eq!(i8080.get_register(Register::A), i8080.read_u8(0x300));
        }
        #[test]
        fn mov() {
            let mut i8080 = i8080!();
            i8080.set_register(Register::A, 0x13);
            i8080.mov(Register::D, Register::A);
            assert_eq!(i8080.get_register(Register::D), i8080.get_register(Register::A));
        }
        #[test]
        fn mov_m() {
            let mut i8080 = i8080!();
            i8080.set_register(Register::A, 0x13);
            i8080.set_register_pair(RegisterPair::H, 0x0300);
            i8080.mov_m(Register::A);
            assert_eq!(i8080.get_register(Register::A), i8080.read_u8(0x300));
        }
        #[test]
        fn inr() {
            let mut i8080 = i8080!();
            i8080.set_register(Register::A, 0b10100010);
            i8080.inr(Register::A);
            assert_eq!(i8080.get_register(Register::A), 0b10100011); 
            assert_eq!(i8080.get_flag(Flag::S), true);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::A), true);
            assert_eq!(i8080.get_flag(Flag::P), true);
        }
        #[test]
        fn inr_m() {
            let mut i8080 = i8080!();
            let location = 0x300;
            i8080.write_u8(location, 0b10100010);
            i8080.set_register_pair(RegisterPair::H, location);
            i8080.inr_m();
            assert_eq!(i8080.read_u8(location), 0b10100011);
            assert_eq!(i8080.get_flag(Flag::S), true);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::A), true);
            assert_eq!(i8080.get_flag(Flag::P), true);
        }
        #[test]
        fn dcr() {
            let mut i8080 = i8080!();
            i8080.set_register(Register::A, 0b10000000);
            i8080.dcr(Register::A);
            assert_eq!(i8080.get_register(Register::A), 0b01111111); 
            assert_eq!(i8080.get_flag(Flag::S), false);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::A), true);
            assert_eq!(i8080.get_flag(Flag::P), false);
        }
        #[test]
        fn dcr_m() {
            let mut i8080 = i8080!();
            let location = 0x300;
            i8080.write_u8(location, 0b10000000);
            i8080.set_register_pair(RegisterPair::H, location);
            i8080.dcr_m();
            assert_eq!(i8080.read_u8(location), 0b01111111);
            assert_eq!(i8080.get_flag(Flag::S), false);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::A), true);
            assert_eq!(i8080.get_flag(Flag::P), false);
        }
        #[test]
        fn rlc() {
            let mut i8080 = i8080!();
            i8080.set_register(Register::A, 0b11110010);
            i8080.rlc();
            assert_eq!(i8080.a, 0b11100101);
            assert_eq!(i8080.get_flag(Flag::C), true);
            i8080.set_register(Register::A, 0b01100111);
            i8080.rlc();
            assert_eq!(i8080.a, 0b11001110);
            assert_eq!(i8080.get_flag(Flag::C), false);
        }

        #[test]
        fn ral() {
            let mut i8080 = i8080!();
            i8080.set_register(Register::A, 0b10110101);
            i8080.ral();
            assert_eq!(i8080.a, 0b01101010);
            assert_eq!(i8080.get_flag(Flag::C), true);
            i8080.ral();
            assert_eq!(i8080.a, 0b11010101);
            assert_eq!(i8080.get_flag(Flag::C), false);
        }
        #[test]
        fn rrc() {
            let mut i8080 = i8080!();
            i8080.set_register(Register::A, 0b11110010);
            i8080.rrc();
            assert_eq!(i8080.a, 0b01111001);
            assert_eq!(i8080.get_flag(Flag::C), false);
            i8080.rrc();
            assert_eq!(i8080.a, 0b10111100);
            assert_eq!(i8080.get_flag(Flag::C), true);
        }
        #[test]
        fn rar() {
            let mut i8080 = i8080!();
            i8080.set_register(Register::A, 0b01101010);
            i8080.set_carry(1);
            i8080.rar();
            assert_eq!(i8080.a, 0b10110101);
            assert_eq!(i8080.get_flag(Flag::C), false);
            i8080.rar();
            assert_eq!(i8080.a, 0b01011010);
            assert_eq!(i8080.get_flag(Flag::C), true);
        }
        #[test]
        fn stc() {
            let mut i8080 = i8080!();
            i8080.stc();
            assert_eq!(i8080.get_flag(Flag::C), true);
        }
        #[test]
        fn cmc() {
            let mut i8080 = i8080!();
            i8080.flags = 0b11000110;
            i8080.stc();
            assert_eq!(i8080.flags, 0b11000111);
        }
        #[test]
        fn cma() {
            let mut i8080 = i8080!();
            i8080.a = 0b01010001;
            i8080.cma();
            assert_eq!(i8080.a, 0b10101110);
        }
        #[test]
        fn daa() {
            let mut i8080 = i8080!();
            i8080.a = 0b10011011;
            i8080.daa();
            assert_eq!(i8080.a, 1);
            assert_eq!(i8080.get_flag(Flag::A), true);
            assert_eq!(i8080.get_flag(Flag::C), true);
        }
        #[test]
        fn add() {
            let mut i8080 = i8080!();
            i8080.d = 0x2E;
            i8080.a = 0x6C;
            i8080.add(Register::D);
            assert_eq!(i8080.a, 0x9A);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), false);
            assert_eq!(i8080.get_flag(Flag::P), true);
            assert_eq!(i8080.get_flag(Flag::S), true);
            assert_eq!(i8080.get_flag(Flag::A), true);
        }
        #[test]
        fn add_m() {
            let mut i8080 = i8080!();
            let location = 0x300;
            i8080.write_u8(location, 0x2E);
            i8080.set_register_pair(RegisterPair::H, location);
            i8080.a = 0x6C;
            i8080.add_m();
            assert_eq!(i8080.a, 0x9A);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), false);
            assert_eq!(i8080.get_flag(Flag::P), true);
            assert_eq!(i8080.get_flag(Flag::S), true);
            assert_eq!(i8080.get_flag(Flag::A), true);
        }
        #[test]
        fn adc() {
            let mut i8080 = i8080!();
            i8080.set_carry(1);
            i8080.c = 0x3D;
            i8080.a = 0x42;
            i8080.adc(Register::C);
            assert_eq!(i8080.a, 0x80);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), false);
            assert_eq!(i8080.get_flag(Flag::P), false);
            assert_eq!(i8080.get_flag(Flag::S), true);
            assert_eq!(i8080.get_flag(Flag::A), true);
        }
        #[test]
        fn adc_m() {
            let mut i8080 = i8080!();
            let location = 0x300;
            i8080.write_u8(location, 0x3D);
            i8080.set_register_pair(RegisterPair::H, location);
            i8080.set_carry(1);
            i8080.a = 0x42;
            i8080.adc_m();
            assert_eq!(i8080.a, 0x80);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), false);
            assert_eq!(i8080.get_flag(Flag::P), false);
            assert_eq!(i8080.get_flag(Flag::S), true);
            assert_eq!(i8080.get_flag(Flag::A), true);
        }
        #[test]
        fn sub() {
            let mut i8080 = i8080!();
            i8080.a = 0x3E;
            i8080.sub(Register::A);
            assert_eq!(i8080.a, 0x0);
            assert_eq!(i8080.get_flag(Flag::Z), true);
            assert_eq!(i8080.get_flag(Flag::C), false);
            assert_eq!(i8080.get_flag(Flag::P), true);
            assert_eq!(i8080.get_flag(Flag::S), false);
        }
        #[test]
        fn sub_m() {
            let mut i8080 = i8080!();
            let location = 0x300;
            i8080.write_u8(location, 0x3E);
            i8080.set_register_pair(RegisterPair::H, location);
            i8080.a = 0x3E;
            i8080.sub_m();
            assert_eq!(i8080.a, 0x0);
            assert_eq!(i8080.get_flag(Flag::Z), true);
            assert_eq!(i8080.get_flag(Flag::C), false);
            assert_eq!(i8080.get_flag(Flag::P), true);
            assert_eq!(i8080.get_flag(Flag::S), false);
        }
        #[test]
        fn sbb() {
            let mut i8080 = i8080!();
            i8080.set_carry(1);
            i8080.l = 0x02;
            i8080.a = 0x04;
            i8080.sbb(Register::L);
            assert_eq!(i8080.a, 0x01);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), false);
            assert_eq!(i8080.get_flag(Flag::P), false);
            assert_eq!(i8080.get_flag(Flag::S), false);
        }  
        #[test]
        fn sbb_m() {
            let mut i8080 = i8080!();
            let location = 0x300;
            i8080.write_u8(location, 0x02);
            i8080.set_register_pair(RegisterPair::H, location);
            i8080.set_carry(1);
            i8080.a = 0x04;
            i8080.sbb_m();
            assert_eq!(i8080.a, 0x01);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), false);
            assert_eq!(i8080.get_flag(Flag::P), false);
            assert_eq!(i8080.get_flag(Flag::S), false);
        }
        #[test]
        fn ana() {
            let mut i8080 = i8080!();
            i8080.a = 0xFC;
            i8080.c = 0x0F;
            i8080.ana(Register::C);
            assert_eq!(i8080.a, 0x0C);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), false);
            assert_eq!(i8080.get_flag(Flag::P), true);
            assert_eq!(i8080.get_flag(Flag::S), false);
        }
        #[test]
        fn ana_m() {
            let mut i8080 = i8080!();
            let location = 0x300;
            i8080.write_u8(location, 0x0F);
            i8080.set_register_pair(RegisterPair::H, location);
            i8080.a = 0xFC;
            i8080.ana_m();
            assert_eq!(i8080.a, 0x0C);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), false);
            assert_eq!(i8080.get_flag(Flag::P), true);
            assert_eq!(i8080.get_flag(Flag::S), false);
        }
        #[test]
        fn xra() {
            let mut i8080 = i8080!();
            i8080.a = 0b10110001;
            i8080.b = 0b11010110;
            i8080.xra(Register::B);
            assert_eq!(i8080.a, 0b01100111);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), false);
            assert_eq!(i8080.get_flag(Flag::P), false);
            assert_eq!(i8080.get_flag(Flag::S), false);
        }
        #[test]
        fn xra_m() {
            let mut i8080 = i8080!();
            let location = 0x300;
            i8080.write_u8(location, 0b11010110);
            i8080.set_register_pair(RegisterPair::H, location);
            i8080.a = 0b10110001;
            i8080.xra_m();
            assert_eq!(i8080.a, 0b01100111);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), false);
            assert_eq!(i8080.get_flag(Flag::P), false);
            assert_eq!(i8080.get_flag(Flag::S), false);
        }
        #[test]
        fn ora() {
            let mut i8080 = i8080!();
            i8080.a = 0x33;
            i8080.c = 0x0F;
            i8080.ora(Register::C);
            assert_eq!(i8080.a, 0x3F);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), false);
            assert_eq!(i8080.get_flag(Flag::P), true);
            assert_eq!(i8080.get_flag(Flag::S), false);
        }
        #[test]
        fn ora_m() {
            let mut i8080 = i8080!();
            let location = 0x300;
            i8080.write_u8(location, 0x0F);
            i8080.set_register_pair(RegisterPair::H, location);
            i8080.a = 0x33;
            i8080.ora_m();
            assert_eq!(i8080.a, 0x3F);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), false);
            assert_eq!(i8080.get_flag(Flag::P), true);
            assert_eq!(i8080.get_flag(Flag::S), false);
        }
        #[test]
        fn cmp() {
            let mut i8080 = i8080!();
            i8080.a = 0x0A;
            i8080.e = 0x05;
            i8080.cmp(Register::E);
            assert_eq!(i8080.a, 0x0A);
            assert_eq!(i8080.e, 0x05);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), false);
            assert_eq!(i8080.get_flag(Flag::P), true);
            assert_eq!(i8080.get_flag(Flag::S), false);

            i8080.a = 0x02;
            i8080.e = 0x05;
            i8080.cmp(Register::E);
            assert_eq!(i8080.a, 0x02);
            assert_eq!(i8080.e, 0x05);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), true);
            assert_eq!(i8080.get_flag(Flag::P), false);
            assert_eq!(i8080.get_flag(Flag::S), true);

            i8080.a = 0b11100101;
            i8080.e = 0x05;
            i8080.cmp(Register::E);
            assert_eq!(i8080.a, 0b11100101);
            assert_eq!(i8080.e, 0x05);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), false);
            assert_eq!(i8080.get_flag(Flag::P), false);
            assert_eq!(i8080.get_flag(Flag::S), true);
        }
        #[test]
        fn cmp_m() {
            let mut i8080 = i8080!();
            let location = 0x300;
            i8080.write_u8(location, 0x05);
            i8080.set_register_pair(RegisterPair::H, location);
            i8080.a = 0x02;
            i8080.cmp_m();
            assert_eq!(i8080.read_u8(location), 0x05);
            assert_eq!(i8080.a, 0x02);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), true);
            assert_eq!(i8080.get_flag(Flag::P), false);
            assert_eq!(i8080.get_flag(Flag::S), true);
        }
        #[test]
        fn adi() {
            let mut i8080 = i8080![0x42];
            i8080.a = 0x14;
            i8080.adi();
            assert_eq!(i8080.a, 0x56);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), false);
            assert_eq!(i8080.get_flag(Flag::P), true);
            assert_eq!(i8080.get_flag(Flag::S), false);
        }
        #[test]
        fn aci() {
            let mut i8080 = i8080![0x42];
            i8080.a = 0x14;
            i8080.set_carry(1);
            i8080.aci();
            assert_eq!(i8080.a, 0x57);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), false);
            assert_eq!(i8080.get_flag(Flag::P), false);
            assert_eq!(i8080.get_flag(Flag::S), false);
        }
        #[test]
        fn sui() {
            let mut i8080 = i8080![0x1];
            i8080.a = 0x00;
            i8080.sui();
            assert_eq!(i8080.a, 0xFF);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), true);
            assert_eq!(i8080.get_flag(Flag::P), true);
            assert_eq!(i8080.get_flag(Flag::S), true);
        }
        #[test]
        fn sbi() {
            let mut i8080 = i8080![0x1];
            i8080.a = 0x00;
            i8080.set_carry(1);
            i8080.sbi();
            assert_eq!(i8080.a, 0xFE);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), true);
            assert_eq!(i8080.get_flag(Flag::P), false);
            assert_eq!(i8080.get_flag(Flag::S), true);
        }
        #[test]
        fn ani() {
            let mut i8080 = i8080![0x0F];
            i8080.a = 0x3A;
            i8080.ani();
            assert_eq!(i8080.a, 0x0A);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), false);
            assert_eq!(i8080.get_flag(Flag::P), true);
            assert_eq!(i8080.get_flag(Flag::S), false);
        }
        #[test]
        fn xri() {
            let mut i8080 = i8080![0x81];
            i8080.a = 0x3B;
            i8080.xri();
            assert_eq!(i8080.a, 0b10111010);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), false);
            assert_eq!(i8080.get_flag(Flag::P), false);
            assert_eq!(i8080.get_flag(Flag::S), true);
        }
        #[test]
        fn ori() {
            let mut i8080 = i8080![0x0F];
            i8080.a = 0xB5;
            i8080.ori();
            assert_eq!(i8080.a, 0xBF);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), false);
            assert_eq!(i8080.get_flag(Flag::P), false);
            assert_eq!(i8080.get_flag(Flag::S), true);
        }
        #[test]
        fn cpi() {
            let mut i8080 = i8080![0x40];
            i8080.a = 0x4A;
            i8080.cpi();
            assert_eq!(i8080.a, 0x4A);
            assert_eq!(i8080.get_flag(Flag::Z), false);
            assert_eq!(i8080.get_flag(Flag::C), false);
            assert_eq!(i8080.get_flag(Flag::P), true);
            assert_eq!(i8080.get_flag(Flag::S), false);
        }
        #[test]
        fn inx() {
            let mut i8080 = i8080!();
            i8080.d = 0x38;
            i8080.e = 0xFF;
            i8080.inx(RegisterPair::D);
            assert_eq!(i8080.d, 0x39);
            assert_eq!(i8080.e, 0x00);
        }
        #[test]
        fn inx_sp() {
            let mut i8080 = i8080!();
            i8080.sp = 0xFFFF;
            i8080.inx_sp();
            assert_eq!(i8080.sp, 0);
        }
        #[test]
        fn dcx() {
            let mut i8080 = i8080!();
            i8080.h = 0x98;
            i8080.l = 0x00;
            i8080.dcx(RegisterPair::H);
            assert_eq!(i8080.h, 0x97);
            assert_eq!(i8080.l, 0xFF);
        }
        #[test]
        fn dcx_sp() {
            let mut i8080 = i8080!();
            i8080.sp = 0xFFFF;
            i8080.dcx_sp();
            assert_eq!(i8080.sp, 0xFFFE);
        }
        #[test]
        fn dad() {
            let mut i8080 = i8080!();
            i8080.set_register_pair(RegisterPair::B, 0x339F);
            i8080.set_register_pair(RegisterPair::H, 0xA17B);
            i8080.dad(RegisterPair::B);
            assert_eq!(i8080.get_register_pair(RegisterPair::H), 0xD51A);
            assert_eq!(i8080.get_flag(Flag::C), false);

            i8080.set_register_pair(RegisterPair::B, 0xFFDD);
            i8080.set_register_pair(RegisterPair::H, 0x0123);
            i8080.dad(RegisterPair::B);
            assert_eq!(i8080.get_register_pair(RegisterPair::H), 0x0100);
            assert_eq!(i8080.get_flag(Flag::C), true);

            i8080.sp = 0x1232;
            i8080.dad_sp();
            assert_eq!(i8080.get_register_pair(RegisterPair::H), 0x1332);
            assert_eq!(i8080.get_flag(Flag::C), false);
        }
    }
}

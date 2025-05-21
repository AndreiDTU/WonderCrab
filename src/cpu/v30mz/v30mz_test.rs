use crate::soc::SoC;
use crate::assert_eq_hex;

use super::*;

#[test]
fn test_0x88_mov_reg_to_mem_8() {
    let mut soc = SoC::new();
    soc.set_wram(vec![
        0x88, 0xC1,             // CL <- AL
        0x88, 0x04,             // WRAM[IX] <- AL
        0x88, 0x06, 0xFE, 0x00, // WRAM[0x00FE] <- AL
    ]);
    soc.get_cpu().AW = 0x1234;
    soc.get_cpu().IX = 0xFF;

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0002);
    assert_eq_hex!(soc.get_cpu().CW, 0x0034);

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0004);
    assert_eq_hex!(soc.get_wram()[0x00FF], 0x34);

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0008);
    assert_eq_hex!(soc.get_wram()[0x00FE], 0x34);
}

#[test]
fn test_0x89_mov_reg_to_mem_16() {
    let mut soc = SoC::new();
    soc.set_wram(vec![
        0x89, 0xC1,             // CW <- AW
        0x89, 0x04,             // WRAM[IX].16 <- AW
        0x89, 0x06, 0xFE, 0x00, // WRAM[0x00FE].16 <- AW
    ]);
    soc.get_cpu().AW = 0x1234;
    soc.get_cpu().IX = 0xFF;

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0002);
    assert_eq_hex!(soc.get_cpu().CW, 0x1234);

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0004);
    assert_eq_hex!(soc.get_wram()[0x00FF], 0x34);
    assert_eq_hex!(soc.get_wram()[0x0100], 0x12);

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0008);
    assert_eq_hex!(soc.get_wram()[0x00FE], 0x34);
    assert_eq_hex!(soc.get_wram()[0x00FF], 0x12);
}

#[test]
fn test_0x8a_mov_mem_to_reg_8() {
    let mut soc = SoC::new();
    soc.set_wram(vec![
        0x8A, 0xC1,             // AL <- CL
        0x8A, 0x04,             // AL <- WRAM[IX] = 0xFF
        0x8A, 0x06, 0xFE, 0x00, // AL <- WRAM[0x00FE] = 0x12
    ]);
    soc.get_cpu().CW = 0x1234;
    soc.get_cpu().IX = 0xFF;
    soc.get_wram()[0x00FF] = 0xFF;
    soc.get_wram()[0x00FE] = 0x12;

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0002);
    assert_eq_hex!(soc.get_cpu().AW, 0x0034);

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0004);
    assert_eq_hex!(soc.get_cpu().AW, 0x00FF);

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0008);
    assert_eq_hex!(soc.get_cpu().AW, 0x0012);
}

#[test]
fn test_0x8b_mov_mem_to_reg_16() {
    let mut soc = SoC::new();
    soc.set_wram(vec![
        0x8B, 0xC1,             // AW <- CW
        0x8B, 0x04,             // AW <- WRAM[IX] = 0xFFFF
        0x8B, 0x06, 0xFE, 0x00, // AW <- WRAM[0x00FE] = 0xFF12
    ]);
    soc.get_cpu().CW = 0x1234;
    soc.get_cpu().IX = 0xFF;
    soc.get_wram()[0x00FF] = 0xFF;
    soc.get_wram()[0x0100] = 0xFF;
    soc.get_wram()[0x00FE] = 0x12;

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0002);
    assert_eq_hex!(soc.get_cpu().AW, 0x1234);

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0004);
    assert_eq_hex!(soc.get_cpu().AW, 0xFFFF);

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0008);
    assert_eq_hex!(soc.get_cpu().AW, 0xFF12);
}

#[test]
fn test_0x8c_mov_seg_to_mem() {
    let mut soc = SoC::new();
    soc.set_wram(vec![
        0x8C, 0xC1,             // CW <- DS1
        0x8C, 0x04,             // WRAM[IX] <- DS1
        0x8C, 0x06, 0xFE, 0x00, // WRAM[0x00FE] <- DS1
    ]);
    soc.get_cpu().DS1 = 0x1234;
    soc.get_cpu().IX = 0xFF;

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0002);
    assert_eq_hex!(soc.get_cpu().CW, 0x1234);

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0004);
    assert_eq_hex!(soc.get_wram()[0x00FF], 0x34);
    assert_eq_hex!(soc.get_wram()[0x0100], 0x12);

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0008);
    assert_eq_hex!(soc.get_wram()[0x00FE], 0x34);
    assert_eq_hex!(soc.get_wram()[0x00FF], 0x12);
}

#[test]
fn test_0x8d_ldea() {
    let mut soc = SoC::new();
    soc.set_wram(vec![
        0x8D, 0x06, 0xCD, 0xAB, // Immediate offset
        0x8D, 0xC1,             // Pointer to CW
        0x8D, 0x40, 0x11,       // BW + IX + 0x11
        0x8D, 0x40, 0xFF,       // BW + IX - 1
        0x8D, 0x80, 0x11, 0x11, // BW + IX + 0x1111
        0x8D, 0x80, 0xFF, 0xFF, // BW + IX - 1
    ]);

    soc.get_cpu().CW = 0x1234;
    soc.get_cpu().BW = 0x5678;
    soc.get_cpu().IX = 0x1111;

    soc.tick();
    assert_eq_hex!(soc.get_cpu().AW, 0xABCD);

    soc.tick();
    assert_eq_hex!(soc.get_cpu().AW, 0x1234);

    soc.tick();
    assert_eq_hex!(soc.get_cpu().AW, 0x679A);

    soc.tick();
    assert_eq_hex!(soc.get_cpu().AW, 0x6788);

    soc.tick();
    assert_eq_hex!(soc.get_cpu().AW, 0x789A);

    soc.tick();
    assert_eq_hex!(soc.get_cpu().AW, 0x6788);
}

#[test]
fn test_0x8e_mov_mem_to_seg() {
    let mut soc = SoC::new();
    soc.set_wram(vec![
        0x8E, 0xC1,             // DS1 <- CW
        0x8E, 0x04,             // DS1 <- WRAM[IX] = 0xFFFF
        0x8E, 0x06, 0xFE, 0x00, // DS1 <- WRAM[0x00FE] = 0xFF12
    ]);
    soc.get_cpu().CW = 0x1234;
    soc.get_cpu().IX = 0xFF;
    soc.get_wram()[0x00FF] = 0xFF;
    soc.get_wram()[0x0100] = 0xFF;
    soc.get_wram()[0x00FE] = 0x12;

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0002);
    assert_eq_hex!(soc.get_cpu().DS1, 0x1234);

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0004);
    assert_eq_hex!(soc.get_cpu().DS1, 0xFFFF);

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0008);
    assert_eq_hex!(soc.get_cpu().DS1, 0xFF12);
}

#[test]
fn test_0x98_cvtbw() {
    let mut cpu = V30MZ::new();
    
    cpu.AW = 0x00FF;
    cpu.current_op = vec![0x98];
    let _ = cpu.execute();
    assert_eq_hex!(cpu.AW, 0xFFFF);

    cpu.AW = 0xFF00;
    cpu.current_op = vec![0x98];
    let _ = cpu.execute();
    assert_eq_hex!(cpu.AW, 0x0000);
}

#[test]
fn test_0x99_cvtwl() {
    let mut cpu = V30MZ::new();

    cpu.AW = 0x8000;
    cpu.current_op = vec![0x99];
    let _ = cpu.execute();
    assert_eq_hex!(cpu.DW, 0xFFFF);

    cpu.AW = 0x7FFF;
    cpu.current_op = vec![0x99];
    let _ = cpu.execute();
    assert_eq_hex!(cpu.DW, 0x0000);
}

#[test]
fn test_0x9e_mov_ah_to_psw() {
    let mut soc = SoC::new();
    soc.set_wram(vec![
        0x9E, // PSW <- AH
    ]);

    soc.get_cpu().AW = 0x55_00;
    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0001);
    assert_eq_hex!(soc.get_cpu().PSW.bits() & 0xFF, 0x55);
}

#[test]
fn test_0x9f_mov_psw_to_ah() {
    let mut soc = SoC::new();
    soc.set_wram(vec![
        0x9F, // AH <- PSW
    ]);

    soc.get_cpu().PSW = CpuStatus::from_bits_truncate(0b1010_1010);
    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0001);
    assert_eq_hex!(soc.get_cpu().AW >> 8, 0xAA);
}

#[test]
fn test_0xa0_mov_dir_mem_to_acc_16() {
    let mut soc = SoC::new();
    soc.set_wram(vec![
        0xA0, 0x34, 0x12, // AL <- [0x1234]
    ]);

    soc.get_cpu().DS0 = 0x0000;
    soc.get_cpu().AW = 0x0000;
    soc.get_wram()[0x1234] = 0xAB;

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0003);
    assert_eq_hex!(soc.get_cpu().AW, 0x00AB);
}

#[test]
fn test_0xa1_mov_dir_mem_to_acc_16() {
    let mut soc = SoC::new();
    soc.set_wram(vec![
        0xA1, 0x34, 0x12, // AW <- [0x1234]
    ]);

    soc.get_cpu().DS0 = 0x0000;
    soc.get_cpu().AW = 0x0000;
    soc.get_wram()[0x1234] = 0xCD;
    soc.get_wram()[0x1235] = 0xAB;

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0003);
    assert_eq_hex!(soc.get_cpu().AW, 0xABCD);
}

#[test]
fn test_0xa2_mov_al_to_dir_mem_8() {
    let mut soc = SoC::new();
    soc.set_wram(vec![
        0xA2, 0x34, 0x12, // [0x1234] <- AL
    ]);

    soc.get_cpu().DS0 = 0x0000;
    soc.get_cpu().AW = 0x00FE;

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0003);
    assert_eq_hex!(soc.get_wram()[0x1234], 0xFE);
}

#[test]
fn test_0xa3_mov_aw_to_dir_mem() {
    let mut soc = SoC::new();
    soc.set_wram(vec![
        0xA3, 0x34, 0x12, // [0x1234] <- AW
    ]);

    soc.get_cpu().DS0 = 0x0000;
    soc.get_cpu().AW = 0xBEEF;

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0003);
    assert_eq_hex!(soc.get_wram()[0x1234], 0xEF);
    assert_eq_hex!(soc.get_wram()[0x1235], 0xBE);
}

#[test]
fn test_0xb0_mov_imm_to_reg_8() {
    let mut soc = SoC::new();
    soc.set_wram(vec![
        0xB0, 0x12, // AL <- 0x12
    ]);

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0002);
    assert_eq_hex!(soc.get_cpu().AW, 0x0012);
}

#[test]
fn test_0xb8_mov_imm_to_reg_16() {
    let mut soc = SoC::new();
    soc.set_wram(vec![
        0xB8, 0x34, 0x12, // AW <- 0x1234
    ]);

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0003);
    assert_eq_hex!(soc.get_cpu().AW, 0x1234);
}

#[test]
fn test_0xc6_mov_imm_to_mem_8() {
    let mut soc = SoC::new();
    soc.set_wram(vec![
        0xC6, 0x06, 0x00, 0x01, 0xAB, // WRAM[0x0100] <- 0xAB
    ]);

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0005);
    assert_eq_hex!(soc.get_wram()[0x0100], 0xAB);
}

#[test]
fn test_0xc7_mov_imm_to_mem_16() {
    let mut soc = SoC::new();
    soc.set_wram(vec![
        0xC7, 0x06, 0x00, 0x01, 0x34, 0x12, // WRAM[0x0100] <- 0x1234
    ]);

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0006);
    assert_eq_hex!(soc.get_wram()[0x0100], 0x34);
    assert_eq_hex!(soc.get_wram()[0x0101], 0x12);
}

#[test]
fn test_0xc4_mov_mem_to_ds1_reg() {
    let mut soc = SoC::new();
    soc.set_wram(vec![
        0xC4, 0x06, 0x00, 0x01, // CW <- 0x1234, DS1 <- 0x5678
    ]);
    soc.get_wram()[0x0100] = 0x34;
    soc.get_wram()[0x0101] = 0x12;
    soc.get_wram()[0x0102] = 0x78;
    soc.get_wram()[0x0103] = 0x56;

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0004);
    assert_eq_hex!(soc.get_cpu().AW, 0x1234);
    assert_eq_hex!(soc.get_cpu().DS1, 0x5678);
}

#[test]
fn test_0xc5_mov_mem_to_ds0_reg() {
    let mut soc = SoC::new();
    soc.set_wram(vec![
        0xC5, 0x06, 0x00, 0x01, // CW <- 0x1234, DS0 <- 0x5678
    ]);
    soc.get_wram()[0x0100] = 0x34;
    soc.get_wram()[0x0101] = 0x12;
    soc.get_wram()[0x0102] = 0x78;
    soc.get_wram()[0x0103] = 0x56;

    soc.tick();
    assert_eq_hex!(soc.get_cpu().PC, 0x0004);
    assert_eq_hex!(soc.get_cpu().AW, 0x1234);
    assert_eq_hex!(soc.get_cpu().DS0, 0x5678);
}

#[test]
fn test_0xe4_in() {
    let mut soc = SoC::new();
    soc.set_wram(vec![0xE4, 0x00]);
    soc.set_io(vec![0xCD, 0xAB]);
    soc.tick();
    assert_eq_hex!(soc.get_cpu().AW, 0x00CD);
}

#[test]
fn test_0xe5_in() {
    let mut soc = SoC::new();
    soc.set_wram(vec![0xE5, 0x00]);
    soc.set_io(vec![0xCD, 0xAB]);
    soc.tick();
    assert_eq_hex!(soc.get_cpu().AW, 0xABCD);
}

#[test]
fn test_0xec_in() {
    let mut soc = SoC::new();
    soc.set_wram(vec![0xEC, 0xFF]);
    soc.set_io(vec![0xCD, 0xAB]);
    soc.get_cpu().DW = 0x00;
    soc.tick();
    assert_eq_hex!(soc.get_cpu().AW, 0x00CD);
}

#[test]
fn test_0xed_in() {
    let mut soc = SoC::new();
    soc.set_wram(vec![0xED, 0xFF]);
    soc.set_io(vec![0xCD, 0xAB]);
    soc.get_cpu().DW = 0x00;
    soc.tick();
    assert_eq_hex!(soc.get_cpu().AW, 0xABCD);
}
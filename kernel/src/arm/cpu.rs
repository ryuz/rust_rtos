#![allow(dead_code)]
#![cfg(target_arch = "arm")]

#[repr(C)]
#[derive(Default)]
pub struct CpuControlBlock {
    pub imask: u32,
}

static mut CPU_CB: CpuControlBlock = CpuControlBlock { imask: 0 };

pub unsafe fn cpu_lock() {
    asm!(
        r#"
            mrs     r0, cpsr                    /* cpsr取得 */
            orr     r0, r0, #(0x40 | 0x80)      /* FビットとIビットを設定 */
            msr     cpsr_c, r0                  /* cpsr設定 */
        "#
    );
}

pub unsafe fn cpu_unlock() {
    let imask = CPU_CB.imask;
    asm!(
        r#"
            mrs     r0, cpsr                    /* cpsr取得 */
            bic     r0, r0, #(0x40 | 0x80)      /* FビットとIビットをクリア */
            orr     r0, r0, {0}                 /* 割込みマスク設定 */
            msr     cpsr_c, r0                  /* cpsr設定 */
        "#,
        in(reg) imask,
    );
}

/// 割り込み待ち
pub unsafe fn wfi() {
    asm!("wfi");
}

///  分岐予測有効化
pub unsafe fn enable_bpredict() {
    asm!(
        r#"
            mrc     p15, 0, r0, c1, c0, 1       /* Read ACTLR */
            bic     r0, r0, #0x1 << 17          /* Clear RSDIS bit 17 to enable return stack */
            bic     r0, r0, #0x1 << 16          /* Clear BP bit 15 and BP bit 16 */
            bic     r0, r0, #0x1 << 15          /* Normal operation, BP is taken from the global history table */
            mcr     p15, 0, r0, c1, c0, 1       /* Write ACTLR */
            dsb
        "#
    );
}

/// 分岐予測無効化
pub unsafe fn disable_bpredict() {
    asm!(
        r#"
            mrc     p15, 0, r0, c1, c0, 1       /* Read ACTLR */
            orr     r0, r0, #0x1 << 17          /* Enable RSDIS bit 17 to disable the return stack */
            orr     r0, r0, #0x1 << 16          /* Clear BP bit 15 and set BP bit 16:*/
            bic     r0, r0, #0x1 << 15          /* Branch always not taken and history table updates disabled*/
            mcr     p15, 0, r0, c1, c0, 1       /* Write ACTLR */
            dsb
        "#
    );
}

/// Iキャッシュ有効化
pub unsafe fn enable_icache() {
    asm!(
        r#"
            mrc     p15, 0, r1, c1, c0, 0   /* Read System Control Register configuration data */
            orr     r1, r1, #0x1 << 12      /* instruction cache enable */
            mcr     p15, 0, r0, c7, c5, 0   /* Invalidate entire instruction cache */
            mcr     p15, 0, r1, c1, c0, 0   /* enabled instruction cache */
            isb
        "#
    );
}

/// Iキャッシュ無効化
pub unsafe fn disable_icache() {
    asm!(
        r#"
            mrc     p15, 0, R1, c1, c0, 0   /* Read System Control Register configuration data */
            bic     r1, r1, #0x1 << 12      /* instruction cache enable */
            mcr     p15, 0, r1, c1, c0, 0   /* disabled instruction cache */
            isb
        "#
    );
}

/// Dキャッシュ有効化
pub unsafe fn enable_dcache() {
    asm!(
        r#"
            mrc     p15, 0, r1, c1, c0, 0   /* Read System Control Register configuration data */
            orr     r1, r1, #0x1 << 2
            dsb
            mcr     p15, 0, r0, c15, c5, 0  /* Invalidate entire data cache */
            mcr     p15, 0, r1, c1, c0, 0   /* enabled data cache */
        "#
    );
}

/// Dキャッシュ無効化
pub unsafe fn disable_dcache() {
    asm!(
        r#"
            mrc     p15, 0, r1, c1, c0, 0   /* Read System Control Register configuration data */
            bic     r1, r1, #0x1 << 2
            dsb
            mcr     p15, 0, r1, c1, c0, 0   /* disabled data cache */
            /* Clean entire data cache. This routine depends on the data cache size. It can be
            omitted if it is known that the data cache has no dirty data. */
        "#
    );
}

/// キャッシュ有効化
pub unsafe fn enable_cache() {
    asm!(
        r#"
            mrc     p15, 0, r1, c1, c0, 0   /* Read System Control Register configuration data */
            orr     r1, r1, #0x1 << 12      /* instruction cache enable */
            orr     r1, r1, #0x1 << 2       /* data cache enable */
            dsb
            mcr     p15, 0, r0, c15, c5, 0  /* Invalidate entire data cache */
            mcr     p15, 0, r0, c7, c5, 0   /* Invalidate entire instruction cache */
            mcr     p15, 0, r1, c1, c0, 0   /* enabled cache RAMs */
            isb
        "#
    );
}

/// キャッシュ無効化
pub unsafe fn disable_cache() {
    asm!(
        r#"
            mrc     p15, 0, r1, c1, c0, 0   /* Read System Control Register configuration data */
            bic     r1, r1, #0x1 << 12      /* instruction cache disable */
            bic     r1, r1, #0x1 << 2       /* data cache disable */
            dsb
            mcr     p15, 0, r1, c1, c0, 0   /* disabled cache RAMs */
            isb
            /* Clean entire data cache. This routine depends on the data cache size. It can be
            omitted if it is known that the data cache has no dirty data */
        "#
    );
}

/// ECC有効化 (必ずキャッシュOFF状態で呼ぶこと)
pub unsafe fn enable_ecc() {
    asm!(
        r#"
            mrc     p15, 0, r1, c1, c0, 1   /* Read Auxiliary Control Register */
            bic     r0, r0, #(0x1 << 5)     /* Generate abort on parity errors, with [5:3]= b000 */
            bic     r0, r0, #(0x1 << 4)
            bic     r0, r0, #(0x1 << 3)
            mcr     p15, 0, r1, c1, c0, 1   /* Write Auxiliary Control Register */
        "#
    );
}

/// ECC無効化 (必ずキャッシュOFF状態で呼ぶこと)
pub unsafe fn disable_ecc() {
    asm!(
        r#"
            mrc     p15, 0, r1, c1, c0, 1   /* Read Auxiliary Control Register */
            orr     r0, r0, #(0x1 << 5)     /* Disable parity checking, with [5:3]= b100 */
            bic     r0, r0, #(0x1 << 4)
            bic     r0, r0, #(0x1 << 3)
            mcr     p15, 0, r1, c1, c0, 1   /* Write Auxiliary Control Register */
        "#
    );
}

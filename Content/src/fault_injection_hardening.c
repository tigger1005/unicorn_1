/*
 * Copyright (c) 2020-2021, Arm Limited. All rights reserved.
 *
 * SPDX-License-Identifier: BSD-3-Clause
 *
 */

#include "bootutil/fault_injection_hardening.h"

#ifdef FIH_ENABLE_CFI
fih_uint fih_cfi_ctr = FIH_UINT_INIT(0u);

fih_uint_ret fih_cfi_get_and_increment(uint8_t cnt)
{
    fih_uint saved_ctr = fih_cfi_ctr;

    if (fih_uint_decode(fih_cfi_ctr) > UINT32_MAX - cnt) {
        /* Overflow */
        FIH_PANIC;
    }

#ifdef FIH_ENABLE_DOUBLE_VARS
    fih_cfi_ctr.val = fih_cfi_ctr.val + cnt;
    fih_cfi_ctr.msk = ((fih_cfi_ctr.msk ^ FIH_UINT_MASK_VALUE) + cnt) ^ FIH_UINT_MASK_VALUE;
#else
    fih_cfi_ctr = fih_uint_encode(fih_uint_decode(fih_cfi_ctr) + cnt);
#endif

    fih_uint_validate(fih_cfi_ctr);
    fih_uint_validate(saved_ctr);

    return saved_ctr;
}

void fih_cfi_validate(fih_uint saved)
{
    volatile int32_t rc = FIH_FALSE;

    rc = fih_uint_eq(saved, fih_cfi_ctr);
    if (rc != FIH_TRUE) {
        FIH_PANIC;
    }
}

void fih_cfi_decrement(void)
{
    if (fih_uint_decode(fih_cfi_ctr) < 1u) {
        FIH_PANIC;
    }

#ifdef FIH_ENABLE_DOUBLE_VARS
    fih_cfi_ctr.val = fih_cfi_ctr.val - 1u;
    fih_cfi_ctr.msk = ((fih_cfi_ctr.msk ^ FIH_UINT_MASK_VALUE) - 1u) ^ FIH_UINT_MASK_VALUE;
#else
    fih_cfi_ctr = fih_uint_encode(fih_uint_decode(fih_cfi_ctr) - 1u);
#endif

    fih_uint_validate(fih_cfi_ctr);
}
#endif /* FIH_ENABLE_CFI */

#ifdef FIH_ENABLE_GLOBAL_FAIL
/* Global failure loop for bootloader code. Uses attribute used to prevent
 * compiler removing due to non-standard calling procedure. Multiple loop jumps
 * used to make unlooping difficult.
 */
__attribute__((noinline))
__attribute__((noreturn))
__attribute__((weak))
void fih_panic_loop(void)
{
#ifdef FAULT_INJECTION_TEST
    do {
        *((volatile unsigned int *)(0xAA01000)) = 0x2;
    } while (1);
#else
  FIH_LABEL("FAILURE_LOOP");
    __asm volatile ("b fih_panic_loop");
    __asm volatile ("b fih_panic_loop");
    __asm volatile ("b fih_panic_loop");
    __asm volatile ("b fih_panic_loop");
    __asm volatile ("b fih_panic_loop");
    __asm volatile ("b fih_panic_loop");
    __asm volatile ("b fih_panic_loop");
    __asm volatile ("b fih_panic_loop");
    __asm volatile ("b fih_panic_loop");
    while (true) {} /* Satisfy noreturn */
#endif
}
#endif /* FIH_ENABLE_GLOBAL_FAIL */

#ifdef FIH_ENABLE_DELAY
void fih_delay_init(void)
{
    /* Implement here */
}

uint8_t fih_delay_random(void)
{
    /* Implement here */

    return 0xFF;
}
#endif /* FIH_ENABLE_DELAY */

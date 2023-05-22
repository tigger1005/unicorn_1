#include "common.h"
#include "fih_mem.h"
#include "utils.h"

void launch_oem_ram_app(void);
int cyboot_read_lcs(uint32_t *lcs, uint32_t checksum);


#define LCS_VIRGIN 0x01230123
#define LCS_SORT  0x12341234
#define LCS_PROVISIONED 0x23452345
#define LCS_NORMAL 0x34563456
#define LCS_NORMAL_NO_SECURE 0x45674567
#define LCS_NORMAL_PROVISIONED 0x56785678
#define LCS_SECURE 0x67896789
#define LCS_RMA_KBR 0x12345678
#define LCS_RMA_KBNR 0x11112222
#define LCS_RMA_KPBR 0x22223333
#define LCS_RMA_KPBSR 0x33334444
#define LCS_RMA 0x44445555
#define LCS_CORRUPTED 0xFEFEFEFE

#define CYBOOT_SUCCESS 0
#define CYBOOT_BOOTROW_CORRUPTED -1

#define GLOBAL_CFI_START_VALUE 0x123B
#define GLOBAL_CFI_END_VALUE (GLOBAL_CFI_START_VALUE - 3)

int main() {
  flash_load_img();
  uint32_t lcs = LCS_CORRUPTED;
  uint32_t checksum = *(uint32_t *) IMG_LOAD_ADDR;
  int res = -1;

  res = cyboot_read_lcs(&lcs, checksum);
  if (res == CYBOOT_SUCCESS && lcs == LCS_RMA) {
    serial_puts("Verification positive path  : OK\n");
    launch_oem_ram_app();
  }
  else {
    serial_puts("Verification negative path : OK\n");
    __SET_SIM_FAILED();
  }
  return 0;
}

/*******************************************************************************
 * Function Name:  launch_oem_ram_app
 *******************************************************************************
 * \brief This function launch CM33 OEM RAM App.
 *
 * \param secure_boot_policy    The policy secure boot value.
 * \param ram_app_start_addr    The start address of RAM App.
 *
 *******************************************************************************/
void launch_oem_ram_app(void) {
  __SET_SIM_SUCCESS();
}


int cyboot_read_lcs(uint32_t *lcs, volatile uint32_t checksum)
{
  int ret = CYBOOT_BOOTROW_CORRUPTED;
  *lcs = LCS_CORRUPTED;

  switch(checksum)
  {
    case LCS_VIRGIN:
    case LCS_SORT:
    case LCS_PROVISIONED:
    case LCS_NORMAL:
    case LCS_NORMAL_NO_SECURE:
    case LCS_NORMAL_PROVISIONED:
    case LCS_SECURE:
        *lcs = checksum;
        ret = CYBOOT_SUCCESS;
        break;
    case LCS_RMA_KBR:
    case LCS_RMA_KBNR:
    case LCS_RMA_KPBR:
    case LCS_RMA_KPBSR:
        if (checksum != LCS_RMA_KBR && checksum != LCS_RMA_KBNR && checksum != LCS_RMA_KPBR && checksum != LCS_RMA_KPBSR)
        {
          FIH_PANIC;
        }
        *lcs = LCS_RMA;
        ret = CYBOOT_SUCCESS;
        break;
    default:
        *lcs = LCS_CORRUPTED;
        ret = CYBOOT_BOOTROW_CORRUPTED;
        break;
  };

  return ret;
}

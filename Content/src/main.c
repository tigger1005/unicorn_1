#include "common.h"
#include "fih_mem.h"
#include "utils.h"
#include <stdint.h>

void launch_oem_ram_app(void);

#define GLOBAL_CFI_START_VALUE 0x123B
#define GLOBAL_CFI_END_VALUE (GLOBAL_CFI_START_VALUE - 3)

int main() {
  flash_load_img();

  if (*(uint32_t *)IMG_LOAD_ADDR == *(uint32_t *)&image_good_val) {
    __SET_SIM_SUCCESS();
  } else {
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
void launch_oem_ram_app(void) { __SET_SIM_SUCCESS(); }
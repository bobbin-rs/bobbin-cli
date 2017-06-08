pub const DEBUG_ERR_OK: u8             = 0x80;
pub const DEBUG_ERR_FAULT: u8          = 0x81;
pub const SWD_AP_WAIT: u8              = 0x10;
pub const SWD_AP_FAULT: u8             = 0x11;
pub const SWD_AP_ERROR: u8             = 0x12;
pub const SWD_AP_PARITY_ERROR: u8      = 0x13;
pub const JTAG_WRITE_ERROR: u8         = 0x0c;
pub const JTAG_WRITE_VERIF_ERROR: u8   = 0x0d;
pub const SWD_DP_WAIT: u8              = 0x14;
pub const SWD_DP_FAULT: u8             = 0x15;
pub const SWD_DP_ERROR: u8             = 0x16;
pub const SWD_DP_PARITY_ERROR: u8      = 0x17;

pub const SWD_AP_WDATA_ERROR: u8       = 0x18;
pub const SWD_AP_STICKY_ERROR: u8      = 0x19;
pub const SWD_AP_STICKYORUN_ERROR: u8  = 0x1a;

pub const CORE_RUNNING: u8             = 0x80;
pub const CORE_HALTED: u8              = 0x81;

pub const GET_VERSION: u8              = 0xF1;
pub const DEBUG_COMMAND: u8            = 0xF2;
pub const DFU_COMMAND: u8              = 0xF3;
pub const SWIM_COMMAND: u8             = 0xF4;
pub const GET_CURRENT_MODE: u8         = 0xF5;
pub const GET_TARGET_VOLTAGE: u8       = 0xF7;

pub const DEV_DFU_MODE: u8             = 0x00;
pub const DEV_MASS_MODE: u8            = 0x01;
pub const DEV_DEBUG_MODE: u8           = 0x02;
pub const DEV_SWIM_MODE: u8            = 0x03;
pub const DEV_BOOTLOADER_MODE: u8      = 0x04;

pub const DFU_EXIT: u8                 = 0x07;

pub const SWIM_ENTER: u8               = 0x00;
pub const SWIM_EXIT: u8                = 0x01;

pub const DEBUG_ENTER_JTAG: u8             = 0x00;
pub const DEBUG_GETSTATUS: u8              = 0x01;
pub const DEBUG_FORCEDEBUG: u8             = 0x02;
pub const DEBUG_APIV1_RESETSYS: u8         = 0x03;
pub const DEBUG_APIV1_READALLREGS: u8      = 0x04;
pub const DEBUG_APIV1_READREG: u8          = 0x05;
pub const DEBUG_APIV1_WRITEREG: u8         = 0x06;
pub const DEBUG_READMEM_32BIT: u8          = 0x07;
pub const DEBUG_WRITEMEM_32BIT: u8         = 0x08;
pub const DEBUG_RUNCORE: u8                = 0x09;
pub const DEBUG_STEPCORE: u8               = 0x0a;
pub const DEBUG_APIV1_SETFP: u8            = 0x0b;
pub const DEBUG_READMEM_8BIT: u8           = 0x0c;
pub const DEBUG_WRITEMEM_8BIT: u8          = 0x0d;
pub const DEBUG_APIV1_CLEARFP: u8          = 0x0e;
pub const DEBUG_APIV1_WRITEDEBUGREG: u8    = 0x0f;
pub const DEBUG_APIV1_SETWATCHPOINT: u8    = 0x10;

pub const DEBUG_ENTER_SWD: u8              = 0xa3;

pub const DEBUG_APIV1_ENTER: u8            = 0x20;
pub const DEBUG_EXIT: u8                   = 0x21;
pub const DEBUG_READCOREID: u8             = 0x22;

pub const DEBUG_APIV2_ENTER: u8            = 0x30;
pub const DEBUG_APIV2_READ_IDCODES: u8     = 0x31;
pub const DEBUG_APIV2_RESETSYS: u8         = 0x32;
pub const DEBUG_APIV2_READREG: u8          = 0x33;
pub const DEBUG_APIV2_WRITEREG: u8         = 0x34;
pub const DEBUG_APIV2_WRITEDEBUGREG: u8    = 0x35;
pub const DEBUG_APIV2_READDEBUGREG: u8     = 0x36;

pub const DEBUG_APIV2_READALLREGS: u8      = 0x3A;
pub const DEBUG_APIV2_GETLASTRWSTATUS: u8  = 0x3B;
pub const DEBUG_APIV2_DRIVE_NRST: u8       = 0x3C;

pub const DEBUG_APIV2_START_TRACE_RX: u8   = 0x40;
pub const DEBUG_APIV2_STOP_TRACE_RX: u8    = 0x41;
pub const DEBUG_APIV2_GET_TRACE_NB: u8     = 0x42;
pub const DEBUG_APIV2_SWD_SET_FREQ: u8     = 0x43;

pub const DEBUG_APIV2_DRIVE_NRST_LOW: u8    = 0x00;
pub const DEBUG_APIV2_DRIVE_NRST_HIGH: u8   = 0x01;
pub const DEBUG_APIV2_DRIVE_NRST_PULSE: u8  = 0x02;

// Temporary Register Constants

pub const SCS_LAR_KEY: u32                = 0xC5ACCE55;
pub const SCS_AIRCR: u32                  = 0xe000ed0c;
pub const SCS_AIRCR_KEY: u32              = (0x05fa << 16);
pub const SCS_AIRCR_VECTCLRACTIVE: u32    = (1 << 1);

pub const DCB_DEMCR: u32                  = 0xE000EDFC;
pub const DCB_DEMCR_TRCENA: u32           = (1 << 24);
pub const DCB_DEMCR_VC_CORERESET: u32     = (1 << 0);  // Enable Reset Vector Catch. This causes a Local reset to halt a running system.

pub const DCB_DHCSR: u32                  = 0xE000EDF0;
pub const DCB_DHCSR_DBGKEY: u32           = (0xA05F << 16);
pub const DCB_DHCSR_C_DEBUGEN: u32        = (1 << 0);
pub const DCB_DHCSR_C_HALT: u32           = (1 << 1);

pub const TPIU_CSPSR: u32                 = 0xe0040004;
pub const TPIU_ACPR: u32                  = 0xE0040010;
pub const TPIU_SPPR: u32                  = 0xE00400F0;
pub const TPIU_FFCR: u32                  = 0xE0040304;
pub const TPIU_SPPR_TXMODE_PARALELL: u32  = 0;
pub const TPIU_SPPR_TXMODE_MANCHESTER: u32 = 1;
pub const TPIU_SPPR_TXMODE_NRZ: u32       = 2;

pub const ITM_LAR: u32                    = 0xe0000fb0;
pub const ITM_TER: u32                    = 0xe0000e00;
pub const ITM_TPR: u32                    = 0xe0000e40;
pub const ITM_TCR: u32                    = 0xe0000e80;
pub const ITM_TCR_SWOENA: u32             = (1 << 4);
pub const ITM_TCR_TXENA: u32              = (1 << 3);
pub const ITM_TCR_SYNCENA: u32            = (1 << 2);
pub const ITM_TCR_TSENA: u32              = (1 << 1);
pub const ITM_TCR_ITMENA: u32             = (1 << 0);

pub const DWT_CTRL: u32                   = 0xE0001000;

// STM32 stuff
pub const DBGMCU_CR: u32                  = 0xe0042004;
pub const DBGMCU_CR_DEBUG_SLEEP: u32      = (1 << 0);
pub const DBGMCU_CR_DEBUG_STOP: u32       = (1 << 1);
pub const DBGMCU_CR_DEBUG_STANDBY: u32    = (1 << 2);
pub const DBGMCU_CR_DEBUG_TRACE_IOEN: u32 = (1 << 5);
pub const DBGMCU_CR_RESERVED_MAGIC_UNKNOWN: u32 = (1 << 8);
pub const DBGMCU_APB1_FZ: u32             = 0xe0042008;
pub const DBGMCU_APB1_FZ_DBG_IWDG_STOP: u32 = (1 << 12);
pub const DBGMCU_IDCODE: u32              = 0xE0042000;

// Cortex-M3 Technical Reference Manual
// Debug Halting Control and Status Register
pub const DHCSR: u32                      = 0xe000edf0;
pub const DCRSR: u32                      = 0xe000edf4;
pub const DCRDR: u32                      = 0xe000edf8;
pub const DBGKEY: u32                     = 0xa05f0000;




# Bobbin-CLI

bobbin-cli is a tool designed to make it easy to build, deploy, test and debug embedded
devices using a unified CLI. bobbin-cli understands Rust's cargo / xargo package managers
but can also work with Make or any other build system.

bobbin-cli has the following main areas of functionality:

-  Device enumeration and selection. bobbin-cli recognizes many types of USB debuggers and loaders
   and allows you to set per-project filters so that it knows which device to use even when
   multiple devices are connected to your computer.

-  Build management. bobbin-cli automatically uses xargo to build your project, and reads the
   command line parameters and your Cargo.toml file to automatically determine the output binary
   to use. You can also use Make, with a required parameter to specify the output binary path.

-  Deployment. For supported devices, bobbin-cli can automatically use the appropriate flash
   loading tool (OpenOCD, JLinkExe, bossac or teensy_cli_loader) to upload the output binary.

-  Testing and Debugging. bobbin-cli can automatically connect to and display the virtual serial
   console of the selected device if available. You can also start an instance of OpenOCD and gdb with
   the output binary produced by the build stage.

For a collection of LED blinking demos for a wide variety of popular development boards,
see [bobbin-blinky](https://github.com/bobbin-rs/bobbin-blinky/).

## Host Platforms

MacOS and Linux are currently supported, Windows support is planned.

## Supported Devices

### Debug Probes

Currently Supported:

- [J-Link](https://www.segger.com/products/debug-probes/j-link/) - including CDC (virtual serial port) support.
- [ST-Link/V2](http://www.st.com/en/development-tools/st-link-v2.html) - V2 and V2.1 devices supported,
including CDC (virtual serial port) and SWO Trace (optional, requires libusb).
- [CMSIS-DAP](https://developer.mbed.org/handbook/CMSIS-DAP) - including CDC (virtual serial port) support.
- [DAPLINK](https://github.com/mbedmicro/DAPLink) - including MSD (mass storage device) and CDC (virtual serial port) support.
- [TI ICDI](http://www.ti.com/tool/stellaris_icdi_drivers) - including CDC (virtual serial port) support.
- [Black Magic Probe](https://github.com/blacksphere/blackmagic) - including CDC (virtual serial port) support.

Coming Soon:

- [PEmicro](http://www.pemicro.com/opensda/)

### Development Boards with Embedded Debug Probes

Boards from the following product families include embedded debug probes that should be supported.

- [STM32 Discovery](http://www.st.com/en/evaluation-tools/stm32-mcu-discovery-kits.html?querycriteria=productId=LN1848) 
- [STM32 Nucleo](http://www.st.com/en/evaluation-tools/stm32-mcu-nucleo.html?querycriteria=productId=LN1847)
- [Freescale FRDM](http://www.nxp.com/products/software-and-tools/hardware-development-tools/freedom-development-boards:FREDEVPLA)
- [TI Launchpad](http://www.ti.com/lsds/ti/tools-software/launchpads/overview/overview.page)
- [NXP S32K144EVB](http://www.nxp.com/products/automotive-products/microcontrollers-and-processors/arm-mcus-and-mpus/s32-processors-microcontrollers/s32k144-evaluation-board:S32K144EVB)
- [Arduino Zero](https://www.arduino.cc/en/Guide/ArduinoZero)

*Note:* Many development boards support OpenSDA, which allows a choice of firmware to be installed. Debug probes may support
CMSIS-DAP, DAPLINK, J-Link and PEMicro firmware variants. Be sure to upgrade to the most recent firmware available, and ensure that
a variant supporting OpenOCD (CMSIS-DAP/DAPLINK) or J-Link is installed. Please see [Upgrading Development Board Firmware](FIRMWARE.md).

### Development Boards with Flash Loader Support

Boards from the following product families use flash loaders that are supported.

- [Feather M0](https://www.adafruit.com/feather)
- [Teensy 3.x and LC](https://www.pjrc.com/teensy/)
- STM32 DFU Bootloader

## Prerequisites

### Build Tools

These tools must be installed in your PATH.

- [GNU ARM Embedded](https://developer.arm.com/open-source/gnu-toolchain/gnu-rm)
- [xargo](https://github.com/japaric/xargo)

### Debugger / Loader Tools

You must have the appropriate tools installed for the debug probes / dev boards that you wish to use.

- [OpenOCD](http://openocd.org) - 0.10 or later required for STLink, DAPLINK, CMSIS-DAP, and TI ICDI debug probes
- [J-Link](https://www.segger.com/downloads/jlink) - required for J-Link debug probes.
- [Bossa](http://www.shumatech.com/web/products/bossa) - required for Arduino and Feather devices
- [Teensy Loader](https://www.pjrc.com/teensy/loader_cli.html) - required for Teensy devices
- [libusb](http://libusb.info) - required for STLink SWO Trace support.
- [dfu-util](http://dfu-util.sourceforge.net) - required for STM32 DFU Bootloader support

### Development Board Firmware

If you are using a development board with embedded debug probe, check that you know what debug firmware you have
installed and that it is up to date. Please see [Development Board Firmware](FIRMWARE.md).

## Installation

*Note:* Only Linux and macOS hosts are supported at this time.

To install from cargo:

```
$ cargo install bobbin-cli
```

To install from github:

```
$ git clone https://github.com/bobbin-rs/bobbin-cli.git
$ cd bobbin-cli
$ cargo install
```

To install with ST-Link SWV Trace support:

```
$ cargo install --features stlink
```

## Usage

The name of the executable is `bobbin`.

### Help

You can display detailed help text by using `bobbin -h`.

### Bobbin Check

Use "bobbin check" to list the version numbers of all Bobbin dependencies. "Not Found" will be displayed if
the dependency is not available.

```
$ bobbin check
      Rust 1.20.0-nightly (83c659ef6 2017-07-18)
     Cargo 0.21.0-nightly (f709c35a3 2017-07-13)
     Xargo 0.3.8
       GCC 5.4.1 20160919 (release) [ARM/embedded-5-branch revision 240496]
   OpenOCD 0.10.0+dev-00092-g77189db (2017-03-01-20:42)
     JLink V6.15c (Compiled Apr 24 2017 19:07:08)
     Bossa 1.7.0
    Teensy 2.1
  dfu-util 0.9
```

Please include the "bobbin check" output when reporting problems.

### Bobbin List

Use "bobbin list" to view all debug probes and development boards connected to your host.

```
$ bobbin list
ID       VID :PID  Vendor                   Product                          Serial Number
4c01a4ad 1366:0105 SEGGER                   J-Link                           000621000000
14a7f5da 03eb:2157 Atmel Corp.              EDBG CMSIS-DAP                   00000000EZE000005574
b7e67550 0483:374b STMicroelectronics       STM32 STLink                     0673FF485550755187121723
a3ef65e3 0483:374b STMicroelectronics       STM32 STLink                     0667FF555654725187073723
cb46720d 1cbe:00fd Texas Instruments        In-Circuit Debug Interface       0F007E1A
8c6bbec5 0d28:0204 ARM                      DAPLink CMSIS-DAP                0260000025414e450049501247e0004e30f1000097969900
f95f4aca 0d28:0204 ARM                      DAPLink CMSIS-DAP                0240000034544e45001b00028aa9001a2011000097969900
c2f3dc42 0483:374b STMicroelectronics       STM32 STLink                     0670FF484957847167071621
$
```

The device ID is a hash of the USB Vendor ID, USB Product ID, and USB Serial Number (if available). "bobbin list" displays
the first eight hex digits of the device ID, and "bobbin info" displays the full 64 bit ID.

### Bobbin Info

To view detailed information about a devices, use the "bobbin info" subcommand.

```
$ bobbin -d 4c01 info
ID               c2f3dc42b4aadc58b6dfa98ce527dd436e3e4fa5
Vendor ID        0483
Product ID       374b
Vendor           STMicroelectronics
Product          STM32 STLink
Serial Number    0670FF484957847167071621
Type             STLinkV21
Loader Type      OpenOCD
Debugger Type    OpenOCD
CDC Device       /dev/cu.usbmodem141413
OpenOCD Serial   hla_serial 0670FF484957847167071621
$
```

If you have more than one connected device, you can select a specific device by using the -d command line
parameter. bobbin-cli will also look for a device filter directive in a YAML configuration file at ./bobbin/config

```
$ cat .bobbin/config
[filter]
device = "c2f3dc42"
```

### Bobbin Build

`bobbin build` runs xargo (by default) or make to build your application. If using xargo, bobbin-cli will
pass through any --target, --bin, --example or --release parameters. 

On completion, bobbin-cli will run `arm-none-eabi-size` on the binary and display the output.

```
$ bobbin build
   Compiling blue-pill v0.1.0 (file:///home/bobbin/bobbin-blinky/blue-pill)
    Finished dev [optimized + debuginfo] target(s) in 0.50 secs
   text	   data	    bss	    dec	    hex	filename
    152	      0	      4	    156	     9c	target/thumbv7em-none-eabihf/debug/blue-pill
$
```

### Bobbin Load

`bobbin load` runs `bobbin build` and then, if successful, load the binary onto the device
using the selected debugger or loader, using objcopy as needed to convert to the appropriate
format. You may include --target, --bin, --example or --release parameters which will be passed
to `bobbin build`.

`bobbin load` will interpret the build parameters as well as the Cargo.toml file to determine
the path to the binary.

```
$ bobbin load
   Compiling blue-pill v0.1.0 (file:///home/bobbin/bobbin-blinky/blue-pill)
    Finished dev [optimized + debuginfo] target(s) in 0.13 secs
   text	   data	    bss	    dec	    hex	filename
    152	      0	      4	    156	     9c	target/thumbv7em-none-eabihf/debug/blue-pill
     Loading target/thumbv7em-none-eabihf/debug/blue-pill.hex
    Complete Successfully flashed device
      Loader Load Complete
$
```


Some devices require manual intervention to enter bootloader mode; you should do this before
running `bobbin load`.

*Note: Many debuggers and loaders require additional configuration*

- OpenOCD: a properly configured openocd.cfg file must be in the current directory. bobbin will add 
the appropriate OpenOCD command line parameters to select the specific device.
- J-Link: you must specify the device type through the --jlink-device command line parameter or
in the .bobbin/config file.
- Teensy Loader: you must specify the device type through the --teensy-mcu command line parameter or
in the .bobbin/config file.

### Bobbin Run

`bobbin run` runs `bobbin load` and then, if successful, open the serial console of the
connected device to display the output. Use Control-C to terminate this console viewer.

If the selected device does not have an associated serial port, that step will be skipped. 

You can use the --console parameter to manually specify a serial device, or --noconsole if 
you do not want run the console viewer at all.

*Note: the serial viewer is currently hard-coded to 115,200 baud*

If bobbin-cli is compiled with support for SWO trace, you can pass the --itm parameter
to display ITM output instead of running the serial console. You will also need to pass
the --itm-target-clock parameter with the target's clock speed.

```
$ bobbin run
   Compiling blue-pill v0.1.0 (file:///home/bobbin/bobbin-hello/blue-pill)
    Finished dev [optimized + debuginfo] target(s) in 0.13 secs
   text	   data	    bss	    dec	    hex	filename
    152	      0	      4	    156	     9c	target/thumbv7em-none-eabihf/debug/blue-pill
     Loading target/thumbv7em-none-eabihf/debug/blue-pill.hex
    Complete Successfully flashed device
      Loader Load Complete
     Console Opening Console
Hello World 1
Hello World 2
Hello World 3
^C
$
```

### Bobbin Test

`bobbin test` runs `bobbin run` and then interprets the serial output, looking for
tags indicating test progress and completion.

```
$ bobbin test
   Compiling frdm-k64f v0.1.0 (file:///home/bobbin/bobbin-boards/frdm-k64f)
    Finished dev [optimized + debuginfo] target(s) in 0.61 secs
   text	   data	    bss	    dec	    hex	filename
   6252	    428	    408	   7088	   1bb0	target/thumbv7em-none-eabihf/debug/frdm-k64f
     Loading target/thumbv7em-none-eabihf/debug/frdm-k64f
    Complete Successfully flashed device
      Loader Load Complete
     Console Opening Console
[start] Running tests for frdm-k64f
[pass] 0
[pass] 1
[pass] 2
[pass] 3
[pass] 4
[done] All tests passed
$
```

`bobbin test` recognizes [start], [pass] and [done] tags, exiting with return code 0. It also recognizes
[fail], [exception], and [panic] tags, which will cause it to exit with return codes 1, 2 or 3. All other
output is ignored.

The test runner will exit with return code 1 if there is a delay of more than 5 seconds between lines
or 15 seconds to complete the entire test. In the future these timeouts will be configurable.

### Additional Subcommands

`bobbin reset` resets the target device.

`bobbin halt` halts the target device, if supported.

`bobbin resume` resumes the target device, if supported.

`bobbin console` starts a console viewer session using the selected device's serial port at a speed
of 115,200.

`bobbin itm` starts an itm viewer session using the selected device.

`bobbin screen` starts a `screen` session using the selected device's serial port at a speed
of 115,200.

`bobbin openocd` starts an `openocd` session using the selected device.

`bobbin jlink` starts a JLinkGDBServer session using the selected device.

`bobbin gdb` starts a GDB session with the current target binary as the executable. For debug probes
that are GDB native, this command will connect directly to the device; for debug probes using
OpenOCD or JLinkGDBServer, you must use `target remote :3333` manually or in a .gdbinit file.

### Using Make instead of Xargo

If you are not using xargo / cargo as your build manager, you have the option of specifying the output binary
path using --binary. You can also use Make as your build manager by using the --make parameter followed by
optional make targets. For instance:

$ bobbin run --make blinky --binary build/blinky.elf

would execute "make blinky" and then continue on success, using build/blinky.elf as the output binary.

## Configuration

### Selecting a device

If you have multiple debug probes connected, you can tell Bobbin which device to use on a per-directory basis.
Bobbin will look for a TOML configuration file in the .bobbin directory (.bobbin/config).

To select a specific device, create a [filter] section with a "device" key that includes the prefix of the
device id. For instance,

```
$ bobbin list
ID       VID :PID  Vendor                   Product                          Serial Number
f95f4aca 0d28:0204 ARM                      DAPLink CMSIS-DAP                0240000034544e45001b00028aa9001a2011000097969900
8c6bbec5 0d28:0204 ARM                      DAPLink CMSIS-DAP                0260000025414e450049501247e0004e30f1000097969900
cb46720d 1cbe:00fd Texas Instruments        In-Circuit Debug Interface       0F007E1A

$ mkdir .bobbin
$ cat > test
[filter]
device = "f95f4aca"
$ bobbin list
ID       VID :PID  Vendor                   Product                          Serial Number
f95f4aca 0d28:0204 ARM                      DAPLink CMSIS-DAP                0240000034544e45001b00028aa9001a2011000097969900
```

### OpenOCD

When using a debug probe / development board that uses OpenCD, you must have an openocd.cfg file in your
project directory that provides the correct configuration for the debugger and device being used.

For instance, for the FRDM-K64F:

```
$ cat openocd.cfg
source [find interface/cmsis-dap.cfg]
source [find target/kx.cfg]
kx.cpu configure -event gdb-attach { reset init }
```

You should be able to run "openocd" and have it successfully connect to the device, assuming you only have
a single debug probe of that type connected:

```
$ openocd
Open On-Chip Debugger 0.10.0+dev-00092-g77189db (2017-03-01-20:42)
Licensed under GNU GPL v2
For bug reports, read
	http://openocd.org/doc/doxygen/bugs.html
Info : auto-selecting first available session transport "swd". To override use 'transport select <transport>'.
Info : add flash_bank kinetis kx.flash
adapter speed: 1000 kHz
none separate
cortex_m reset_config sysresetreq
Info : CMSIS-DAP: SWD  Supported
Info : CMSIS-DAP: Interface Initialised (SWD)
Info : CMSIS-DAP: FW Version = 1.0
Info : SWCLK/TCK = 0 SWDIO/TMS = 1 TDI = 0 TDO = 0 nTRST = 0 nRESET = 1
Info : CMSIS-DAP: Interface ready
Info : clock speed 1000 kHz
Info : SWD DPIDR 0x2ba01477
Info : MDM: Chip is unsecured. Continuing.
Info : kx.cpu: hardware has 6 breakpoints, 4 watchpoints
^C
$
```

Bobbin will invoke OpenOCD with additional command line parameters specifying the USB serial number
of the device to open.

### J-Link

J-Link debug probes require a device identfier that specifies the target MCU. You must specify this by using the
--jlink-device=&lt;JLINK-DEVICE&gt; command line parameter or by adding a jlink-device key to the [loader] section
of your .bobbin/config file:

```
[loader]
jlink-device = "S32K144"
```

You may view a list of devices at [J-Link - Supported Devices](https://www.segger.com/products/debug-probes/j-link/technology/cpus-and-devices/j-link-supported-devices/).

### Teensy Loader

teensy_loader_cli requires an additional command line parameter --teensy-mcu=&lt;MCU&gt; that tells it the exact MCU being used. You
will need to add a teensy-mcu key to the [loader] section of your .bobbin/config file:

```
[loader]
teensy-mcu = "mk20dx256" # Teensy 3.2
```
```
[loader]
teensy-mcu = "mk64fx512" # Teensy 3.5
```
```
[loader]
teensy-mcu = "mk66fx1m0" # Teensy 3.6
```
```
[loader]
teensy-mcu = "mkl26z64" # Teensy LC
```

Use 'teensy_loader_cli --list-mcus' to view a list of supported MCUs.

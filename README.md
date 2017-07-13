# Bobbin-CLI

bobbin-cli (bobbin) is a command line tool for automating your embedded development workflow.

bobbin-cli (bobbin) is a tool designed to make it easy to build, deploy, test and debug embedded
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

If you have only a single debugging device connected to your computer, you should be able to view
it using "bobbin list", which should produce output like this:

```
$ bobbin list
ID       VID :PID  Vendor                   Product                          Serial Number
c2f3dc42 0483:374b STMicroelectronics       STM32 STLink                     0670FF484957847167071621
```

or if you have multiple devices connected:

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
```

Most subcommands will take a global parameter "-d" to specify a specific device ID from the first column. You
may use a unique prefix of the ID - for instance 4c01 instead of 4c01a4ad.

To view additional information, you may use the "info" subcommand:

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
```

Note that for this device, bobbin-cli has identified the virtual serial port and also knows the proper
OpenOCD --command parameter to force use of this specific device.

bobbin-cli will also look for a device filter directive in a YAML configuration file at ./bobbin/config

```
$ cat .bobbin/config
[filter]
device = "c2f3dc42"
```

To build and run a Rust embedded application, simply use "bobbin run" with optional--target, --bin,
--example and --release parameters, just as you would use xargo or xargo directly. bobbin-cli will
use these parameters as well as the local .cargo/config and Cargo.toml file to determine the path of
the output file. It will then execute the appropriate flash loader application for your device (OpenOCD, 
JLinkExe, bossac or teensy_loader_cli), using objcopy as needed to convert to the required format.

Some devices require manual intervention to enter bootloader mode.

By default, if your selected debugger has a detected virtual serial port, bobbin-cli will connect to that
serial port [NOTE: currently hard-coded to 115,200 baud] and display all output. Use Control-C to terminate
this console viewer. You can use the --console parameter to manually specify a serial device, or
--noconsole if you do not want run the console viewer at all.

Finally, you may occasionally need to use OpenOCD (or some other GDB remote debugger) and GDB to debug
a problem. "bobbin openocd" will automatically start OpenOCD with the appropriate parameters to
connect to your specific device, and (in a separate window) "bobbin gdb" will build your application
and then run arm-none-eabi-gdb with the appropiate output binary path as the first parameter. You may wish
to have a .gdbinit file that automatically connects to your local OpenOCD instance.

If you are not using xargo / cargo as your build manager, you have the option of specifying the output binary
path using --binary. You can also use Make as your build manager by using the --make parameter followed by
optional make targets. For instance:

$ bobbin run --make blinky --binary build/blinky.elf

would execute "make blinky" and then continue on success, using build/blinky.elf as the output binary.

## Installation

<!--
To install from cargo:

```
$ cargo install bobbin-cli
```
-->

To install from github:

```
$ git clone https://github.com/bobbin-rs/bobbin-cli.git
$ cd bobbin-cli
$ cargo install bobbin-cli
```
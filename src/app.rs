use clap::{Arg, App, SubCommand};

const ABOUT: &'static str = "
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
it using \"bobbin list\", which should produce output like this:

$ bobbin list
ID       VID :PID  Vendor                   Product                          Serial Number
c2f3dc42 0483:374b STMicroelectronics       STM32 STLink                     0670FF484957847167071621

or you may multiple devices connected:

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

Most subcommands will take a global parameter \"-d\" to specify a specific device ID from the first column. You
may use a unique prefix of the ID - for instance 4c01 instead of 4c01a4ad.

To view additional information, you may use the \"info\" subcommand:

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

Note that for this device, bobbin-cli has identified the virtual serial port and also knows the proper
OpenOCD --command parameter to force use of this specific device.

bobbin-cli will also look for a device filter directive in a YAML configuration file at ./bobbin/config

$ cat .bobbin/config
[filter]
device = \"c2f3dc42\"

To build and run a Rust embedded application, simply use \"bobbin run\" with optional--target, --bin,
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
a problem. \"bobbin openocd\" will automatically start OpenOCD with the appropriate parameters to
connect to your specific device, and (in a separate window) \"bobbin gdb\" will build your application
and then run arm-none-eabi-gdb with the appropiate output binary path as the first parameter. You may wish
to have a .gdbinit file that automatically connects to your local OpenOCD instance.

If you are not using xargo / cargo as your build manager, you have the option of specifying the output binary
path using --binary. You can also use Make as your build manager by using the --make parameter followed by
optional make targets. For instance:

$ bobbin run --make blinky --binary build/blinky.elf

would execute \"make blinky\" and then continue on success, using build/blinky.elf as the output binary.
";

pub fn app() -> App<'static, 'static> {
    App::new("bobbin")
        .version("0.1")
        .author("Jonathan Soo <jcsoo@agora.com>")
        .about(ABOUT)
        .arg(Arg::with_name("verbose").long("verbose").short("v").help("Displays verbose output"))
        .arg(Arg::with_name("quiet").long("quiet").short("q").help("Suppress verbose output"))
        .arg(Arg::with_name("config").long("config").short("c").help("Specify the bobbin config file path"))
        .arg(Arg::with_name("device").long("device").short("d").takes_value(true).help("Specify a device ID prefix for filtering"))
        // .arg(Arg::with_name("vendor-id").long("vendor-id").takes_value(true))
        // .arg(Arg::with_name("product-id").long("product-id").takes_value(true))
        // .arg(Arg::with_name("serial-number").long("serial-number").takes_value(true))
        .subcommand(SubCommand::with_name("list")
            .arg(Arg::with_name("all").long("all").help("Display all USB devices"))
            .about("Display a list of debug devices")
        )
        .subcommand(SubCommand::with_name("info").about("Display detailed information about selected debug devices"))
        .subcommand(SubCommand::with_name("build")
            .arg(Arg::with_name("target").long("target").takes_value(true).help("Pass a --target parameter to xargo"))
            .arg(Arg::with_name("bin").long("bin").takes_value(true).help("Pass a --bin parameter to xargo"))
            .arg(Arg::with_name("example").long("example").takes_value(true).help("Pass a --example parameter to xargo"))
            .arg(Arg::with_name("release").long("release").help("Pass a --release parameter to xargo"))
            .arg(Arg::with_name("features").long("features").takes_value(true).help("Pass a --features parameter to xargo"))
            .arg(Arg::with_name("xargo").long("xargo").help("Use xargo for the build"))
            .arg(Arg::with_name("make").long("make").takes_value(true).multiple(true).min_values(0)
                .help("Use make for the build, optionally providing additional parameters")
            )
            .about("Build an application using xargo or make.")
        )
        .subcommand(SubCommand::with_name("load")
            .arg(Arg::with_name("binary").long("binary").takes_value(true).help("Specify the path of the binary file to load."))
            .arg(Arg::with_name("target").long("target").takes_value(true).help("Pass a --target parameter to xargo"))
            .arg(Arg::with_name("bin").long("bin").takes_value(true).help("Pass a --bin parameter to xargo"))
            .arg(Arg::with_name("example").long("example").takes_value(true).help("Pass a --example parameter to xargo"))
            .arg(Arg::with_name("release").long("release").help("Pass a --release parameter to xargo"))
            .arg(Arg::with_name("features").long("features").takes_value(true).help("Pass a --features parameter to xargo"))
            .arg(Arg::with_name("xargo").long("xargo").help("Use xargo for the build"))
            .arg(Arg::with_name("make").long("make").takes_value(true).multiple(true).min_values(0)
                .help("Use make for the build, optionally providing additional parameters")            
            )
            .arg(Arg::with_name("no-build").long("no-build").help("Don't build before attempting to load."))
            .about("Load an application onto the selected device after a successful build.")
        )
        .subcommand(SubCommand::with_name("run")
            .arg(Arg::with_name("binary").long("binary").takes_value(true).help("Specify the path of the binary file to load."))
            .arg(Arg::with_name("target").long("target").takes_value(true).help("Pass a --bin parameter to xargo"))
            .arg(Arg::with_name("bin").long("bin").takes_value(true).help("Pass a --bin parameter to xargo"))
            .arg(Arg::with_name("example").long("example").takes_value(true).help("Pass a --example parameter to xargo"))
            .arg(Arg::with_name("release").long("release").help("Pass a --release parameter to xargo"))
            .arg(Arg::with_name("features").long("features").takes_value(true).help("Pass a --features parameter to xargo"))
            .arg(Arg::with_name("xargo").long("xargo").help("Use xargo for the build"))
            .arg(Arg::with_name("make").long("make").takes_value(true).multiple(true).min_values(0)
                .help("Use make for the build, optionally providing additional parameters")                        
            )
            .arg(Arg::with_name("no-build").long("no-build").help("Don't build before attempting to load."))
            .arg(Arg::with_name("console").long("console").min_values(0).max_values(1)
                .help("Specify the path to the serial device for the selected device.")
            )
            .arg(Arg::with_name("noconsole").long("no-console").help("Don't attempt to open a serial console after running."))
            .arg(Arg::with_name("itm").long("itm").help("Display the ITM trace output after running."))
            .arg(Arg::with_name("itm-target-clock").long("itm-target-clock").min_values(0).max_values(1)
                .help("Set the ITM Target's Clock Speed"))
            .about("Load and run an application on the selected device after a successful build.")
        )
        .subcommand(SubCommand::with_name("halt").about("Halt the selected device."))
        .subcommand(SubCommand::with_name("resume")
            .arg(Arg::with_name("console").long("console").min_values(0).max_values(1)
                .help("Specify the path to the serial device for the selected device."))
            .arg(Arg::with_name("noconsole").long("no-console")
                .help("Don't attempt to open a serial console after resuming.")
            )
            .about("Resume the selected device.")
        )
        .subcommand(SubCommand::with_name("reset")
            .arg(Arg::with_name("run").long("run").help("Run the device after reset."))
            .arg(Arg::with_name("halt").long("halt").help("Halt the device after reset."))
            .arg(Arg::with_name("init").long("init").help("Initialize the device after reset."))
            .arg(Arg::with_name("console").long("console").min_values(0).max_values(1)
                .help("Specify the path to the serial device for the selected device.")
            )
            .arg(Arg::with_name("noconsole").long("no-console")
                .help("Don't attempt to open a serial console after resuming.")
            )
            .about("Reset the selected device.")
        )
        .subcommand(SubCommand::with_name("console")
            .arg(Arg::with_name("console").long("console").min_values(0).max_values(1)
                .help("Specify the path to the serial device for the selected device.")
            )
            .about("View the serial output of the selected device.")
        )
        .subcommand(SubCommand::with_name("itm")
            .arg(Arg::with_name("itm-target-clock").long("itm-target-clock").help("Set the ITM Target's Clock Speed"))
            .about("View the ITM output of the selected device.")
        )
        .subcommand(SubCommand::with_name("screen")
            .arg(Arg::with_name("console").long("console").min_values(0).max_values(1)
                .help("Specify the path to the serial device for the selected device.")
            )
            .about("Connect to the serial port of the selected device using screen.")
        )        
        .subcommand(SubCommand::with_name("openocd")
            .about("Start OpenOCD for the selected device")
        )
        .subcommand(SubCommand::with_name("gdb")
            .arg(Arg::with_name("binary").long("binary").takes_value(true).help("Specify the path of the binary file to load."))
            .arg(Arg::with_name("target").long("target").takes_value(true).help("Pass a --bin parameter to xargo"))
            .arg(Arg::with_name("bin").long("bin").takes_value(true).help("Pass a --bin parameter to xargo"))
            .arg(Arg::with_name("example").long("example").takes_value(true).help("Pass a --example parameter to xargo"))
            .arg(Arg::with_name("release").long("release").help("Pass a --release parameter to xargo"))
            .arg(Arg::with_name("features").long("features").takes_value(true).takes_value(true).help("Pass a --features parameter to xargo"))
            .arg(Arg::with_name("xargo").long("xargo").help("Use xargo for the build"))
            .arg(Arg::with_name("make").long("make").takes_value(true).multiple(true).min_values(0)
                .help("Use make for the build, optionally providing additional parameters")                        
            )
            .arg(Arg::with_name("no-build").long("no-build").help("Don't build before attempting to load."))
            .about("Start gdb using the build output as the target.")
        )
        //.subcommand(SubCommand::with_name("objdump"))
}
# Development Board Firmware

## Overview

Development boards with integrated debug probes are cheap, convenient and common. They also
have their own complexity - integrated debug probes have MCUs which may be more powerful than
the target MCUs, and have firmware and bootloaders of their own.

## Architecture

Most USB-connected development boards produced today have a similar architecture.

### Target MCU and Peripherals

The Target MCU is the device that the development board is designed around; vendors typically
will have development boards for each major variant of the MCUs that they produce.

Depending on the MCU and the vendor, there may be additional peripherals and connectors
connected to the MCU. Most development boards will include at least one LED and button and
provide access to most device pins through convenient headers.

### Embedded Debug Probe

Elsewhere on the development board will be an Embedded Debug Probe, a small MCU that is usually
connected to an on-board USB micro or USB mini jack. The debug probe typically includes a 
power supply connected to the USB jack that may also be used to power the Target MCU, 
indicator LEDs, and a reset button.

The debug probe will be connected to the Target MCU though a debugging interface such as JTAG
or SWD, which it will use to control and program the Target MCU. It will also have a pin to control
the RESET line of the Target MCU. In many cases, the debug probe will also be connected to
a set of Target MCU UART TX / RX pins that can serve as a serial console.

In some cases some of these pins may be brought out to headers so that they can be used to
debug external devices. The debug pins of the Target MCU and the debug probe MCU itself may separately
be brought out to headers so that an external debug probe can be used to debug the Target MCU
and the embedded debug probe.

The embedded debug probe will have its own firmware stored in flash memory to run an application that allows the debug probe to communicate through the USB port and with the Target MCU. We'll refer to this
as the *Debug Application*. The *Debug Application* will usually provide some kind of low or high
level debugging interface, sometimes a virtual USB mass storage device for drag-and-drop firmware
uploads to the Target MCU, and sometimes a virtual USB serial port connected to the Target MCU
serial port.

There will also need to be a *Bootloader* to load the *Debug Application* into the debug probe's flash memory. Some common MCUs have built-in bootloaders using a protocol such as DFU that can be used for this
purpose; many other debug probes use a specification called OpenSDA which provides a way to upload
firmware by making the debug probe appear as a USB mass storage device. OpenSDA in particular provides
a standard platform that allows different types of debug applications to be installed, one at a time.

## Debug Probe Platforms

There are two major debug probe platforms being used today. 

### ST-Link

ST-Link is used in the popular STM32 Discovery and Nucleo boards, as well as in a large number of
standalone debug probes including cheap clones. While not an official standard, ST-Link is based
on a STM32F103 MCU.

ST-Link debug probes are updating using DFU, but the firmware upload is encrypted. ST provides a
[Firmware Upgrade Utility](http://www.st.com/en/embedded-software/stsw-link007.html) to update their boards with the latest firmware. 

J-Link also provides a Windows-only [STLinkReflash](https://www.segger.com/products/debug-probes/j-link/models/other-j-links/st-link-on-board/) utility
that allows installing the J-Link debug application on ST-Link debug probes.

### OpenSDA

OpenSDA is an open debug probe platform based around a standard hardware interface and USB debug
protocol. OpenSDA provides a bootloader that allows installing debug applications via drag-and-drop
to a virtual USB drive. In many cases the bootloader itself can be upgraded by using drag-and-drop.

The most common debug applications running on OpenSDA devices are

- [DAPLINK](https://github.com/mbedmicro/DAPLink) (formerly [CMSIS-DAP](https://developer.mbed.org/handbook/CMSIS-DAP))
- [J-Link](https://www.segger.com/downloads/jlink)
- [PEMicro](http://www.pemicro.com/opensda/)

## Debug Applications

*Debug Applications* run on the debug probe MCU and are responsible for implementing the USB debug
interface, virtual USB serial ports, and virtual USB mass storage devices for drag-and-drop firmware
uploads. Each debug application uses its own USB debug protocol which needs to be supported by
software running on the debugging host.

### ST-Link

ST-Link is the debug application used by STMicroelectronics on their popular Discover and Nucleo
development boards and on their standalone ST-Link debuggers. Most devices are running ST-Link/V2
or ST-Link/V2-1 and also support virtual serial ports and USB mass storage device protocol for
drag-and-drop programming.

ST-Link/V2 and ST-Link/V2-1 are supported by OpenOCD.

### DAPLINK

[DAPLINK](https://github.com/mbedmicro/DAPLink) (the successor to CMSIS-DAP) is an open source
debug application that runs on a wide variety of devices supporting the OpenSDA specification. It
is found in NXP / Freescale products such as the popular FRDM development boards, as well as many
others. Many debug probes support virtual serial ports and USB mass storage for drag-and-drop programming.

DAPLINK is supported by OpenOCD using the CMSIS-DAP protocol.

### J-Link

[J-Link](https://www.segger.com/products/debug-probes/j-link/) is an extremely popular propietary debug
application that runs on J-Link standalone debug probes as well as a wide variety of development boards.
Segger provides downloadable firmware that can be installed on OpenSDA boards, LPC-Link2 boards and even
ST-Link development boards. Depending on the specific version, virtual serial ports and USB mass storage
for drag-and-drop programming may be available.

J-Link is supported by OpenOCD and by J-Link's own software.


### PEMicro

[PEMicro](http://www.pemicro.com/opensda/) is a proprietary debug application that runs on PEMicro
standalone debug probes and on a wide range of OpenSDA devices. It can support virtual serial ports
as well as USB mass storage for drag-and-drop programming.

PEMicro is not currently supported by OpenOCD (is osbdm currently available?) but PEMicro
does provide [GDB Server for ARM Devices](http://www.pemicro.com/products/product_viewDetails.cfm?product_id=15320151).
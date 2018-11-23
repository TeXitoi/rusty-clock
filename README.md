# rusty-clock [![Build status](https://travis-ci.org/TeXitoi/rusty-clock.svg?branch=master)](https://travis-ci.org/TeXitoi/rusty-clock)

An alarm clock with environment stats in pure rust.

![fromt](images/front.jpg)
![back](images/back.jpg)

## Description

This alarm clock is programed in bare metal rust (no OS). It features pressure, temperature, humidity, monophonic alarm on a e-paper display.

## Hardware

The hardware used in this project is
- a [blue pill board](https://wiki.stm32duino.com/index.php?title=Blue_Pill) featuring a STM32F103C8 microcontroller (20KiB RAM, 64 KiB flash, ARM Cortex M3 @72MHz);
- a [WaveShare e-paper display](https://www.waveshare.com/wiki/2.9inch_e-Paper_Module);
- a [BME280 sensor](https://www.bosch-sensortec.com/bst/products/all_products/bme280) for temperature, humidity and pressure;
- a passive buzzer driven by PWM;
- 4 [16mm self-reset buttons](https://www.aliexpress.com/item/16mm-Metal-Push-Button-Switch-IP67-Waterproof-Nickel-plated-brass-press-button-Self-reset-1NO-High/32867071630.html) (cancel, previous, next, OK);
- a [3D printed case](cad/);
- USB power plug;
- micro USB cable;
- wires and soldering set.

You also need a [ST-Linkv2](https://www.aliexpress.com/wholesale?SearchText=stlink+v2) to flash and debug.

Everything can be purchased on [Aliexpress](https://www.aliexpress.com/) for less that 50$ (without the 3D printed case).

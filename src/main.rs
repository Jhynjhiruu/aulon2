#![feature(let_chains)]

use std::fs::{read, write};

use anyhow::Result;
use bbrdb::{scan_devices, CardStats, GlobalHandle};
use byte_unit::Byte;
use chrono::{DateTime, Local};
use rustyline::{error::ReadlineError, DefaultEditor};

const PROG_NAME: &str = "aulon2";
const PROG_VER: &str = "0.0.1";

#[derive(Default)]
pub struct CliContext {
    player: Option<GlobalHandle>,
}

fn main() -> Result<()> {
    println!("{PROG_NAME} v{PROG_VER}");
    let mut rl = DefaultEditor::new()?;
    let mut context = CliContext::default();
    match scan_devices() {
        Ok(players) => {
            if players.len() == 1 {
                context.player = Some(GlobalHandle::new(&players[0])?)
            }
        }
        Err(e) => return Err(e.into()),
    };
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                let command = line.split(' ').collect::<Vec<_>>();

                if command.is_empty() {
                    continue;
                }

                match command[0] {
                    "" => continue,

                    "h" => {
                        println!(
                            "Commands:

    l                  - List available BB Players
    s device           - Select BB Player <device>

    B                  - Initialise USB connection to the selected console
    I                  - Request the console's unique BBID
    H value            - Set LED (0, 1 = off; 2 = on; 3 = flashing)
    ;S hash_file       - Sign the SHA-1 hash in [hash_file] using ECDSA
    J [time]           - Set console clock to PC's current time, or [time] if given (note: RFC3339 format)
    L                  - List all games currently on the console
    F file             - Dump the current filesystem block to [file]
    X blkno nand spare - Read one block and its spare data from the console to [nand] and [spare]
    Y blkno nand spare - Write one block and its spare data from [nand] and [spare] to the console
    C                  - Print statistics about the console's NAND
    Q                  - Close USB connection to the console

    1 [nand, spare]    - Dump the console's NAND to 'nand.bin' and 'spare.bin', or [nand] and [spare] if both are provided
    3 file             - Read [file] from the console
    4 file             - Write [file] to the console
    5                  - List all files currently on the console
    6 file             - Delete [file] from the console
    7 from to          - Rename [from] to [to]

    h                  - Print this help
    ?                  - Print copyright and licensing information
    q                  - Quit {PROG_NAME}"
                        )
                    }
                    "?" => {
                        println!(
                            "{PROG_NAME} v{PROG_VER}
Copyright © 2023, 2024 Jhynjhiruu (https://github.com/Jhynjhiruu)
{PROG_NAME} is licensed under the GPL v3 (or any later version).

{PROG_NAME} and libbbrdb based on aulon by Jbop; copyright notice reproduced here:

aulon © 2018, 2019, 2020 Jbop (https://github.com/jbop1626)
aulon is licensed under the GPL v3 (or any later version).

Portions Copyright (c) 2012-2018 Mike Ryan
Originally released under the MIT license

libusb is licensed under the LGPL v2.1 (or any later version)
Copyright (c) 2001 Johannes Erdfelt <johannes@erdfelt.com>
Copyright (c) 2007 - 2009 Daniel Drake <dsd@gentoo.org>
Copyright (c) 2010 - 2012 Peter Stuge <peter@stuge.se>
Copyright (c) 2008 - 2016 Nathan Hjelm <hjelmn@users.sourceforge.net>
Copyright (c) 2009 - 2013 Pete Batard <pete@akeo.ie>
Copyright (c) 2009 - 2013 Ludovic Rousseau <ludovic.rousseau@gmail.com>
Copyright (c) 2010 - 2012 Michael Plante <michael.plante@gmail.com>
Copyright (c) 2011 - 2013 Hans de Goede <hdegoede@redhat.com>
Copyright (c) 2012 - 2013 Martin Pieuchot <mpi@openbsd.org>
Copyright (c) 2012 - 2013 Toby Gray <toby.gray@realvnc.com>
Copyright (c) 2013 - 2018 Chris Dickens <christopher.a.dickens@gmail.com>

See the included file LIBUSB_AUTHORS.txt for more."
                        )
                    }

                    "l" => {
                        let players = scan_devices()?;
                        for player in players {
                            println!("{player:?}");
                        }
                    }
                    "s" => {
                        if let Some(player) = &mut context.player {
                            if player.initialised() {
                                eprintln!("Device already opened! Please close it with 'Q' before selecting a new device.");
                                continue;
                            }
                            let _ = player.Close();
                            context.player = None;
                        }
                        if command.len() < 2 {
                            eprintln!("'s' requires an argument, 'device'. Type 'h' for a list of commands and their arguments.");
                            continue;
                        }
                        let device: usize = match command[1].parse() {
                            Ok(d) => d,
                            Err(e) => {
                                eprintln!("{e}");
                                continue;
                            }
                        };
                        let players = scan_devices()?;
                        let player = match players.get(device) {
                            Some(p) => p,
                            None => {
                                eprintln!("Invalid selection: {device}");
                                continue;
                            }
                        };
                        context.player = Some(GlobalHandle::new(player)?);
                        println!("Selected player {device} successfully");
                    }

                    "B" => {
                        if let Some(player) = &mut context.player {
                            match player.Init() {
                                Ok(_) => println!("Init success"),
                                Err(e) => {
                                    eprintln!("{e}")
                                }
                            }
                        } else {
                            eprintln!("No console selected. Have you used the 'l' and 's' commands to select a console?");
                        }
                    }
                    "I" => {
                        if let Some(player) = &mut context.player {
                            match player.GetBBID() {
                                Ok(bbid) => println!("BBID: {bbid:04X}"),
                                Err(e) => {
                                    eprintln!("{e}")
                                }
                            }
                        } else {
                            eprintln!("No console selected. Have you used the 'l' and 's' commands to select a console?");
                        }
                    }
                    "H" => {
                        if let Some(player) = &mut context.player {
                            if command.len() < 2 {
                                eprintln!("'H' requires an argument, 'value'. Type 'h' for a list of commands and their arguments.");
                                continue;
                            }
                            let value: u32 = match command[1].parse() {
                                Ok(v) => v,
                                Err(e) => {
                                    eprintln!("{e}");
                                    continue;
                                }
                            };
                            match player.SetLED(value) {
                                Ok(_) => println!("SetLED success"),
                                Err(e) => {
                                    eprintln!("{e}")
                                }
                            }
                        } else {
                            eprintln!("No console selected. Have you used the 'l' and 's' commands to select a console?");
                        }
                    }
                    "S" => {
                        eprintln!("Unimplemented");
                    }
                    "J" => {
                        if let Some(player) = &mut context.player {
                            let time = if command.len() < 2 {
                                Local::now().into()
                            } else if let Ok(dt) = DateTime::parse_from_rfc3339(command[1]) {
                                dt
                            } else {
                                eprintln!("Invalid time; 'J' requires a date given in RFC 3339 format, or none to use the current local time. Type 'h' for a list of commands and their arguments.");
                                continue;
                            };
                            match player.SetTime(time) {
                                Ok(_) => println!("SetTime success"),
                                Err(e) => {
                                    eprintln!("{e}")
                                }
                            }
                        } else {
                            eprintln!("No console selected. Have you used the 'l' and 's' commands to select a console?");
                        }
                    }
                    "L" => {
                        if let Some(player) = &mut context.player {
                            match player.ListFiles() {
                                Ok(files) => {
                                    for (filename, size) in files {
                                        if filename.ends_with(".rec") || filename.ends_with(".app")
                                        {
                                            println!(
                                                "{:>12}: {:>7}",
                                                filename,
                                                Byte::from_bytes(size as u128)
                                                    .get_appropriate_unit(true)
                                                    .format(0)
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("{e}")
                                }
                            }
                        } else {
                            eprintln!("No console selected. Have you used the 'l' and 's' commands to select a console?");
                        }
                    }
                    "F" => {
                        if let Some(player) = &mut context.player {
                            if command.len() < 2 {
                                eprintln!("'F' requires an argument, 'file'. Type 'h' for a list of commands and their arguments.");
                                continue;
                            }
                            match player.DumpCurrentFS() {
                                Ok(fs) => match write(command[1], fs) {
                                    Ok(_) => println!("DumpCurrentFS success"),
                                    Err(e) => {
                                        eprintln!("{e}")
                                    }
                                },
                                Err(e) => {
                                    eprintln!("{e}")
                                }
                            }
                        } else {
                            eprintln!("No console selected. Have you used the 'l' and 's' commands to select a console?");
                        }
                    }
                    "X" => {
                        if let Some(player) = &mut context.player {
                            if command.len() < 4 {
                                eprintln!("'X' requires three arguments, 'blkno', 'nand' and 'spare'. Type 'h' for a list of commands and their arguments.");
                                continue;
                            }
                            let blk_num: u32 = match command[1].parse() {
                                Ok(v) => v,
                                Err(e) => {
                                    eprintln!("{e}");
                                    continue;
                                }
                            };
                            let (nand, spare) = match player.ReadSingleBlock(blk_num) {
                                Ok(ns) => ns,
                                Err(e) => {
                                    eprintln!("{e}");
                                    continue;
                                }
                            };
                            match write(command[2], nand) {
                                Ok(_) => {}
                                Err(e) => {
                                    eprintln!("{e}")
                                }
                            }
                            match write(command[3], spare) {
                                Ok(_) => {
                                    println!("ReadSingleBlock success")
                                }
                                Err(e) => {
                                    eprintln!("{e}")
                                }
                            }
                        } else {
                            eprintln!("No console selected. Have you used the 'l' and 's' commands to select a console?");
                        }
                    }
                    #[cfg(not(feature = "writing"))]
                    "Y" => {
                        eprintln!("This version of {PROG_NAME} was built without support for writing; rebuild with `-F writing` to use this command.")
                    }
                    #[cfg(feature = "writing")]
                    "Y" => {
                        if let Some(player) = &mut context.player {
                            if command.len() < 4 {
                                eprintln!("'Y' requires three arguments, 'blkno', 'nand' and 'spare'. Type 'h' for a list of commands and their arguments.");
                                continue;
                            }
                            let blk_num: u32 = match command[1].parse() {
                                Ok(v) => v,
                                Err(e) => {
                                    eprintln!("{e}");
                                    continue;
                                }
                            };
                            let nand = match read(command[2]) {
                                Ok(n) => n,
                                Err(e) => {
                                    eprintln!("{e}");
                                    continue;
                                }
                            };
                            let spare = match read(command[3]) {
                                Ok(s) => s,
                                Err(e) => {
                                    eprintln!("{e}");
                                    continue;
                                }
                            };
                            match player.WriteSingleBlock(blk_num, &nand, &spare) {
                                Ok(_) => {
                                    println!("WriteSingleBlock success")
                                }
                                Err(e) => {
                                    eprintln!("{e}");
                                }
                            };
                        } else {
                            eprintln!("No console selected. Have you used the 'l' and 's' commands to select a console?");
                        }
                    }
                    "C" => {
                        if let Some(player) = &context.player {
                            match player.CardStats() {
                                Ok(CardStats{free, used, bad, seqno}) =>
                                    println!("Free: {free} ({})\nUsed: {used} ({})\nBad: {bad} ({})\nSequence Number: {seqno}", 
                                        Byte::from_bytes((free * 0x4000) as u128).get_appropriate_unit(true),
                                        Byte::from_bytes((used * 0x4000) as u128).get_appropriate_unit(true),
                                        Byte::from_bytes((bad * 0x4000) as u128).get_appropriate_unit(true)),
                                Err(e) => {
                                    eprintln!("{e}")
                                }
                            }
                        } else {
                            eprintln!("No console selected. Have you used the 'l' and 's' commands to select a console?");
                        }
                    }
                    "Q" => {
                        if let Some(player) = &mut context.player {
                            match player.Close() {
                                Ok(_) => println!("Close success"),
                                Err(e) => {
                                    eprintln!("{e}")
                                }
                            }
                            context.player = None;
                        } else {
                            eprintln!("No console selected. Have you used the 'l' and 's' commands to select a console?");
                        }
                    }

                    "1" => {
                        if let Some(player) = &context.player {
                            let (nand_filename, spare_filename) = if command.len() < 3 {
                                ("nand.bin", "spare.bin")
                            } else {
                                (command[1], command[2])
                            };
                            let (nand, spare) = match player.DumpNANDSpare() {
                                Ok(ns) => {
                                    println!("DumpNAND success");
                                    ns
                                }
                                Err(e) => {
                                    eprintln!("{e}");
                                    continue;
                                }
                            };
                            match write(nand_filename, nand) {
                                Ok(_) => {}
                                Err(e) => {
                                    eprintln!("{e}")
                                }
                            }
                            match write(spare_filename, spare) {
                                Ok(_) => {}
                                Err(e) => {
                                    eprintln!("{e}")
                                }
                            }
                        } else {
                            eprintln!("No console selected. Have you used the 'l' and 's' commands to select a console?");
                        }
                    }
                    "3" => {
                        if let Some(player) = &mut context.player {
                            if command.len() < 2 {
                                eprintln!("'3' requires an argument, 'file'. Type 'h' for a list of commands and their arguments.");
                                continue;
                            }

                            let file = match player.ReadFile(command[1]) {
                                Ok(f) => match f {
                                    Some(d) => {
                                        println!("ReadFile success");
                                        d
                                    }
                                    None => {
                                        eprintln!("File {} not found", command[1]);
                                        continue;
                                    }
                                },
                                Err(e) => {
                                    eprintln!("{e}");
                                    continue;
                                }
                            };

                            match write(command[1], file) {
                                Ok(_) => {}
                                Err(e) => {
                                    eprintln!("{e}")
                                }
                            }
                        } else {
                            eprintln!("No console selected. Have you used the 'l' and 's' commands to select a console?");
                        }
                    }
                    #[cfg(not(feature = "writing"))]
                    "4" => {
                        eprintln!("This version of {PROG_NAME} was built without support for writing; rebuild with `-F writing` to use this command.")
                    }
                    #[cfg(feature = "writing")]
                    "4" => {
                        if let Some(player) = &mut context.player {
                            if command.len() < 2 {
                                eprintln!("'4' requires an argument, 'file'. Type 'h' for a list of commands and their arguments.");
                                continue;
                            }

                            let f = read(command[1]).map_err(std::io::Error::into);
                            match f.and_then(|data| player.WriteFile(&data, command[1])) {
                                Ok(_) => println!("WriteFile success"),
                                Err(e) => {
                                    eprintln!("{e}");
                                    continue;
                                }
                            }
                        } else {
                            eprintln!("No console selected. Have you used the 'l' and 's' commands to select a console?");
                        }
                    }
                    "5" => {
                        if let Some(player) = &mut context.player {
                            match player.ListFiles() {
                                Ok(files) => {
                                    for (filename, size) in files {
                                        println!(
                                            "{:>12}: {:>7}",
                                            filename,
                                            Byte::from_bytes(size as u128)
                                                .get_appropriate_unit(true)
                                                .format(0)
                                        );
                                    }
                                }
                                Err(e) => {
                                    eprintln!("{e}")
                                }
                            }
                        } else {
                            eprintln!("No console selected. Have you used the 'l' and 's' commands to select a console?");
                        }
                    }
                    #[cfg(not(feature = "writing"))]
                    "6" => {
                        eprintln!("This version of {PROG_NAME} was built without support for writing; rebuild with `-F writing` to use this command.")
                    }
                    #[cfg(feature = "writing")]
                    "6" => {
                        if let Some(player) = &mut context.player {
                            if command.len() < 2 {
                                eprintln!("'6' requires an argument, 'file'. Type 'h' for a list of commands and their arguments.");
                                continue;
                            }

                            match player.DeleteFile(command[1]) {
                                Ok(_) => println!("DeleteFile success"),
                                Err(e) => {
                                    eprintln!("{e}");
                                    continue;
                                }
                            };
                        } else {
                            eprintln!("No console selected. Have you used the 'l' and 's' commands to select a console?");
                        }
                    }
                    #[cfg(not(feature = "writing"))]
                    "7" => {
                        eprintln!("This version of {PROG_NAME} was built without support for writing; rebuild with `-F writing` to use this command.")
                    }
                    #[cfg(feature = "writing")]
                    "7" => {
                        if let Some(player) = &mut context.player {
                            if command.len() < 2 {
                                eprintln!("'7' requires two arguments, 'from' and 'to'. Type 'h' for a list of commands and their arguments.");
                                continue;
                            }

                            let (from, to) = (command[1], command[2]);
                            match player.RenameFile(from, to) {
                                Ok(ns) => {
                                    println!("RenameFile success");
                                    ns
                                }
                                Err(e) => {
                                    eprintln!("{e}");
                                    continue;
                                }
                            };
                        } else {
                            eprintln!("No console selected. Have you used the 'l' and 's' commands to select a console?");
                        }
                    }

                    "q" => {
                        break;
                    }

                    _ => {
                        eprintln!("Invalid command. Type 'h' for a list of valid commands.")
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {}
            Err(ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("{e}");
                return Err(e.into());
            }
        }
    }

    Ok(())
}

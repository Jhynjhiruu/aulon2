#![feature(let_chains)]

use std::fs::write;

use anyhow::Result;
use bb::BBPlayer;
use byte_unit::Byte;
use rustyline::{error::ReadlineError, DefaultEditor};

const PROG_NAME: &str = "aulon2";
const PROG_VER: &str = "0.0.1";

#[derive(Default)]
pub struct CliContext {
    player: Option<BBPlayer>,
}

fn main() -> Result<()> {
    println!("{PROG_NAME} v{PROG_VER}");
    let mut rl = DefaultEditor::new()?;
    let mut context = CliContext::default();
    match BBPlayer::get_players() {
        Ok(players) => {
            if players.len() == 1 {
                context.player = Some(BBPlayer::new(&players[0])?)
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
    J                  - Set console clock to PC's current time
    L                  - List all files currently on the console
    F file             - Dump the current filesystem block to [file]
    1 [nand, spare]    - Dump the console's NAND to 'nand.bin' and 'spare.bin', or [nand] and [spare] if both are provided
    X blkno nand spare - Read one block and its spare data from the console to [nand] and [spare]
    3 file             - Read [file] from the console
    ;C                 - Print statistics about the console's NAND
    Q                  - Close USB connection to the console

    h                  - Print this help
    ?                  - Print copyright and licensing information
    q                  - Quit {PROG_NAME}"
                        )
                    }
                    "?" => {
                        println!(
                            "{PROG_NAME} v{PROG_VER}
Copyright © 2023 Jhynjhiruu (https://github.com/Jhynjhiruu)
{PROG_NAME} is licensed under the GPL v3 (or any later version).

{PROG_NAME} and libbb based on aulon by Jbop; copyright notice reproduced here:

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
                        let players = BBPlayer::get_players()?;
                        for player in players {
                            println!("{player:?}");
                        }
                    }
                    "s" => {
                        if let Some(player) = &mut context.player && player.initialised() {
                            eprintln!("Device already opened! Please close it with 'Q' before selecting a new device.");
                            continue;
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
                        let players = BBPlayer::get_players()?;
                        let player = match players.get(device) {
                            Some(p) => p,
                            None => {
                                eprintln!("Invalid selection: {device}");
                                continue;
                            }
                        };
                        context.player = Some(BBPlayer::new(player)?);
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
                            match player.SetTime() {
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
                                        println!(
                                            "{:>12}: {:>7}",
                                            filename,
                                            Byte::from_bytes(size.into())
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
                    "1" => {
                        if let Some(player) = &mut context.player {
                            let (nand_filename, spare_filename) = if command.len() < 3 {
                                ("nand.bin", "spare.bin")
                            } else {
                                (command[1], command[2])
                            };
                            let (nand, spare) = match player.DumpNAND() {
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
                    "C" => {
                        eprintln!("Unimplemented")
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
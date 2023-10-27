#![no_std]
#![no_main]

mod utils;
mod app_ui {
    pub mod menu;
}

use core::str::from_utf8;
use nanos_sdk::buttons::ButtonEvent;
use nanos_sdk::ecc::{Secp256k1, SeedDerive};
use nanos_sdk::io;
use nanos_sdk::io::SyscallError;
use nanos_ui::ui;

use nanos_ui::bitmaps::{EYE, VALIDATE_14, CROSSMARK};

use app_ui::menu::ui_menu_main;

nanos_sdk::set_panic!(nanos_sdk::exiting_panic);

pub const BIP32_PATH: [u32; 5] = nanos_sdk::ecc::make_bip32_path(b"m/44'/535348'/0'/0/0");

/// Display public key in two separate
/// message scrollers
fn show_pubkey() {
    let pubkey = Secp256k1::derive_from_path(&BIP32_PATH).public_key();
    match pubkey {
        Ok(pk) => {
            {
                let hex0 = utils::to_hex(&pk.as_ref()[1..33]).unwrap();
                let m = from_utf8(&hex0).unwrap();
                ui::MessageScroller::new(m).event_loop();
            }
            {
                let hex1 = utils::to_hex(&pk.as_ref()[33..65]).unwrap();
                let m = from_utf8(&hex1).unwrap();
                ui::MessageScroller::new(m).event_loop();
            }
        }
        Err(_) => ui::popup("Error"),
    }
}

/// Basic nested menu. Will be subject
/// to simplifications in the future.
#[allow(clippy::needless_borrow)]
fn menu_example() {
    loop {
        match ui::Menu::new(&[&"PubKey", &"Infos", &"Back", &"Exit App"]).show() {
            0 => show_pubkey(),
            1 => loop {
                match ui::Menu::new(&[&"Copyright", &"Authors", &"Back"]).show() {
                    0 => ui::popup("2020 Ledger"),
                    1 => ui::popup("???"),
                    _ => break,
                }
            },
            2 => return,
            3 => nanos_sdk::exit_app(0),
            _ => (),
        }
    }
}

/// This is the UI flow for signing, composed of a scroller
/// to read the incoming message, a panel that requests user
/// validation, and an exit message.
fn sign_ui(message: &[u8]) -> Result<Option<([u8; 72], u32, u32)>, SyscallError> {
    let hex = utils::to_hex(message).map_err(|_| SyscallError::Overflow)?;
    let m = from_utf8(&hex).map_err(|_| SyscallError::InvalidParameter)?;
    let my_field = [ui::Field {
        name: "Data",
        value: m,
    }];

    let my_review = ui::MultiFieldReview::new(
        &my_field,
        &["Review ","Transaction"],
        Some(&EYE),
        "Approve",
        Some(&VALIDATE_14),
        "Reject",
        Some(&CROSSMARK),
    );

    if my_review.show() {
        let signature = Secp256k1::derive_from_path(&BIP32_PATH)
            .deterministic_sign(message)
            .map_err(|_| SyscallError::Unspecified)?;
        ui::popup("Done !");
        Ok(Some(signature))
    } else {
        ui::popup("Cancelled");
        Ok(None)
    }
}

#[no_mangle]
extern "C" fn sample_pending() {
    let mut comm = io::Comm::new();

    loop {
        ui::SingleMessage::new("Pending").show();
        match comm.next_event::<Ins>() {
            io::Event::Button(ButtonEvent::RightButtonRelease) => break,
            _ => (),
        }
    }
    loop {
        ui::SingleMessage::new("Ledger review").show();
        match comm.next_event::<Ins>() {
            io::Event::Button(ButtonEvent::BothButtonsRelease) => break,
            _ => (),
        }
    }
}

#[no_mangle]
extern "C" fn sample_main() {
    let mut comm = io::Comm::new();

    loop {
        // Wait for either a specific button push to exit the app
        // or an APDU command
        match ui_menu_main(&mut comm) {
            io::Event::Command(ins) => match handle_apdu(&mut comm, ins.into()) {
                Ok(()) => comm.reply_ok(),
                Err(sw) => comm.reply(sw),
            },
            _ => (),
        }
    }
}

#[repr(u8)]
enum Ins {
    GetPubkey,
    Sign,
    Menu,
    Exit,
}

impl From<io::ApduHeader> for Ins {
    fn from(header: io::ApduHeader) -> Ins {
        match header.ins {
            2 => Ins::GetPubkey,
            3 => Ins::Sign,
            4 => Ins::Menu,
            0xff => Ins::Exit,
            _ => panic!(),
        }
    }
}

use nanos_sdk::io::Reply;

fn handle_apdu(comm: &mut io::Comm, ins: Ins) -> Result<(), Reply> {
    if comm.rx == 0 {
        return Err(io::StatusWords::NothingReceived.into());
    }

    match ins {
        Ins::GetPubkey => {
            let pk = Secp256k1::derive_from_path(&BIP32_PATH)
                .public_key()
                .map_err(|x| Reply(0x6eu16 | (x as u16 & 0xff)))?;
            comm.append(pk.as_ref());
        }
        Ins::Sign => {
            let out = sign_ui(comm.get_data()?)?;
            if let Some((signature_buf, length, _)) = out {
                comm.append(&signature_buf[..length as usize])
            }
        }
        Ins::Menu => menu_example(),
        Ins::Exit => nanos_sdk::exit_app(0),
    }
    Ok(())
}

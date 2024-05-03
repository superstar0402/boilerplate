/*****************************************************************************
 *   Ledger App Boilerplate Rust.
 *   (c) 2023 Ledger SAS.
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 *****************************************************************************/
use crate::handlers::sign_tx::Tx;
use crate::utils::concatenate;
use crate::AppSW;

#[cfg(not(any(target_os = "stax", target_os = "flex")))]
use ledger_device_sdk::ui::bitmaps::{CROSSMARK, EYE, VALIDATE_14};
#[cfg(not(any(target_os = "stax", target_os = "flex")))]
use ledger_device_sdk::ui::gadgets::{Field, MultiFieldReview};

#[cfg(any(target_os = "stax", target_os = "flex"))]
use crate::settings::Settings;
#[cfg(any(target_os = "stax", target_os = "flex"))]
use include_gif::include_gif;
#[cfg(any(target_os = "stax", target_os = "flex"))]
use ledger_device_sdk::nbgl::{Field, NbglGlyph, NbglReview};

use numtoa::NumToA;

const MAX_COIN_LENGTH: usize = 10;

/// Displays a transaction and returns true if user approved it.
///
/// This method can return [`AppSW::TxDisplayFail`] error if the coin name length is too long.
///
/// # Arguments
///
/// * `tx` - Transaction to be displayed for validation
pub fn ui_display_tx(tx: &Tx) -> Result<bool, AppSW> {
    // Generate string for amount
    let mut numtoa_buf = [0u8; 20];
    let mut value_buf = [0u8; 20 + MAX_COIN_LENGTH + 1];

    let value_str = concatenate(
        &[tx.coin, " ", tx.value.numtoa_str(10, &mut numtoa_buf)],
        &mut value_buf,
    )
    .map_err(|_| AppSW::TxDisplayFail)?; // Fails if value_buf is too small

    // Generate destination address string in hexadecimal format.
    let mut to_str = [0u8; 42];
    to_str[..2].copy_from_slice("0x".as_bytes());
    hex::encode_to_slice(tx.to, &mut to_str[2..]).unwrap();
    to_str[2..].make_ascii_uppercase();

    // Define transaction review fields
    let my_fields = [
        Field {
            name: "Amount",
            value: value_str,
        },
        Field {
            name: "Destination",
            value: core::str::from_utf8(&to_str).unwrap(),
        },
        Field {
            name: "Memo",
            value: tx.memo,
        },
    ];

    // Create transaction review
    #[cfg(not(any(target_os = "stax", target_os = "flex")))]
    {
        let my_review = MultiFieldReview::new(
            &my_fields,
            &["Review ", "Transaction"],
            Some(&EYE),
            "Approve",
            Some(&VALIDATE_14),
            "Reject",
            Some(&CROSSMARK),
        );

        Ok(my_review.show())
    }

    #[cfg(any(target_os = "stax", target_os = "flex"))]
    {
        // Load glyph from 64x64 4bpp gif file with include_gif macro. Creates an NBGL compatible glyph.
        const FERRIS: NbglGlyph = NbglGlyph::from_include(include_gif!("crab_64x64.gif", NBGL));
        // Create NBGL review. Maximum number of fields and string buffer length can be customised
        // with constant generic parameters of NbglReview. Default values are 32 and 1024 respectively.
        let mut review: NbglReview = NbglReview::new()
            .titles(
                "Please review transaction",
                "To send CRAB",
                "Sign transaction\nto send CRAB",
            )
            .glyph(&FERRIS);

        // If first setting switch is disabled do not display the transaction memo
        let settings = Settings::default();
        if settings.get_element(0) == 0 {
            return Ok(review.show(&my_fields[0..2]));
        } else {
            return Ok(review.show(&my_fields));
        }
    }
}

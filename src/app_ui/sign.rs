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

use core::str::from_utf8;

use crate::handlers::sign_tx::Tx;
use crate::utils::concatenate;
use ledger_device_ui_sdk::bitmaps::{CROSSMARK, EYE, VALIDATE_14};
use ledger_device_ui_sdk::ui::{Field, MultiFieldReview};
use numtoa::NumToA;

const MAX_COIN_LENGTH: usize = 10;

pub fn ui_display_tx(tx: &Tx) -> bool {
    // Generate string for amount
    let mut numtoa_buf = [0u8; 20];
    let mut value_buf = [0u8; 20 + MAX_COIN_LENGTH + 1];

    if let Ok(value_str) = concatenate(
        &[tx.coin, &" ", tx.value.numtoa_str(10, &mut numtoa_buf)],
        &mut value_buf,
    ) {
        // Generate destination address string in hexadecimal format.
        let mut to_str = [0u8; 42];
        to_str[..2].copy_from_slice("0x".as_bytes());
        hex::encode_to_slice(tx.to, &mut to_str[2..]).unwrap();

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
        let my_review = MultiFieldReview::new(
            &my_fields,
            &["Review ", "Transaction"],
            Some(&EYE),
            "Approve",
            Some(&VALIDATE_14),
            "Reject",
            Some(&CROSSMARK),
        );

        my_review.show()
    } else {
        // Coin name too long, concatenation buffer was too small.
        return false;
    }
}

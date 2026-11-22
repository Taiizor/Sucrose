//! Pure helpers for the tray icon: picking an entry out of an ICO file and fitting a tooltip into the limit of the
//! Windows notification area. They have no OS dependency so every OS can test them.

/// The longest tooltip the Windows notification area keeps: `NOTIFYICONDATAW::szTip` holds 128 UTF-16 units and
/// the last one is the terminating NUL.
pub const MAX_TOOLTIP_UNITS: usize = 127;

/// Size of the ICONDIR header: reserved, type and image count, 2 bytes each.
const HEADER_LEN: usize = 6;
/// Size of one ICONDIRENTRY.
const ENTRY_LEN: usize = 16;

/// The image data of the ICO entry closest to `wanted_px` (ties go to the higher bit depth), or `None` when the
/// bytes are not an ICO file or no entry points inside them. The returned slice is what `CreateIconFromResourceEx`
/// expects: a DIB (or PNG) without the file header.
pub fn pick_ico_entry(bytes: &[u8], wanted_px: u32) -> Option<&[u8]> {
    let header = bytes.get(..HEADER_LEN)?;
    let reserved = u16::from_le_bytes([header[0], header[1]]);
    let kind = u16::from_le_bytes([header[2], header[3]]);
    let count = usize::from(u16::from_le_bytes([header[4], header[5]]));
    if reserved != 0 || kind != 1 || count == 0 {
        return None;
    }

    let mut best: Option<(u32, std::cmp::Reverse<u16>, &[u8])> = None;
    for index in 0..count {
        let start = HEADER_LEN + index * ENTRY_LEN;
        let Some(entry) = bytes.get(start..start + ENTRY_LEN) else {
            break;
        };
        // A width of 0 stands for 256.
        let width = if entry[0] == 0 { 256 } else { u32::from(entry[0]) };
        let bit_count = u16::from_le_bytes([entry[6], entry[7]]);
        let size = u32::from_le_bytes([entry[8], entry[9], entry[10], entry[11]]) as usize;
        let offset = u32::from_le_bytes([entry[12], entry[13], entry[14], entry[15]]) as usize;
        let Some(data) = offset.checked_add(size).and_then(|end| bytes.get(offset..end)) else {
            continue;
        };
        if data.is_empty() {
            continue;
        }
        let key = (width.abs_diff(wanted_px), std::cmp::Reverse(bit_count));
        if best
            .as_ref()
            .is_none_or(|(distance, depth, _)| key < (*distance, *depth))
        {
            best = Some((key.0, key.1, data));
        }
    }
    best.map(|(_, _, data)| data)
}

/// `text` cut at a character boundary so it is at most [`MAX_TOOLTIP_UNITS`] UTF-16 units long.
pub fn truncate_tooltip(text: &str) -> String {
    let mut units = 0;
    let mut end = text.len();
    for (index, character) in text.char_indices() {
        units += character.len_utf16();
        if units > MAX_TOOLTIP_UNITS {
            end = index;
            break;
        }
    }
    text[..end].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An ICO with one entry per `(width byte, bit count, payload)`, payloads stored right after the directory.
    fn ico(entries: &[(u8, u16, &[u8])]) -> Vec<u8> {
        let mut bytes = vec![0, 0, 1, 0];
        bytes.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        let mut offset = HEADER_LEN + entries.len() * ENTRY_LEN;
        for (width, bits, payload) in entries {
            bytes.extend_from_slice(&[*width, *width, 0, 0, 1, 0]);
            bytes.extend_from_slice(&bits.to_le_bytes());
            bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            bytes.extend_from_slice(&(offset as u32).to_le_bytes());
            offset += payload.len();
        }
        for (_, _, payload) in entries {
            bytes.extend_from_slice(payload);
        }
        bytes
    }

    #[test]
    fn the_entry_closest_to_the_wanted_size_wins() {
        let file = ico(&[(16, 32, b"small"), (32, 32, b"large")]);

        assert_eq!(pick_ico_entry(&file, 16), Some(&b"small"[..]));
        assert_eq!(pick_ico_entry(&file, 32), Some(&b"large"[..]));
        assert_eq!(pick_ico_entry(&file, 28), Some(&b"large"[..]));
    }

    #[test]
    fn a_tie_goes_to_the_higher_bit_depth() {
        let file = ico(&[(16, 8, b"eight"), (16, 32, b"thirty-two")]);

        assert_eq!(pick_ico_entry(&file, 16), Some(&b"thirty-two"[..]));
    }

    #[test]
    fn a_width_of_zero_means_256() {
        let file = ico(&[(16, 32, b"small"), (0, 32, b"huge")]);

        assert_eq!(pick_ico_entry(&file, 256), Some(&b"huge"[..]));
    }

    #[test]
    fn truncated_or_foreign_input_gives_none() {
        let file = ico(&[(16, 32, b"small"), (32, 32, b"large")]);

        assert_eq!(pick_ico_entry(&[], 16), None);
        assert_eq!(pick_ico_entry(&file[..3], 16), None);
        assert_eq!(pick_ico_entry(&file[..HEADER_LEN + 5], 16), None);
        // The directory survives but both images are cut off.
        assert_eq!(pick_ico_entry(&file[..HEADER_LEN + 2 * ENTRY_LEN], 16), None);
        assert_eq!(pick_ico_entry(b"\x89PNG\r\n\x1a\n and more bytes", 16), None);
    }

    #[test]
    fn an_entry_pointing_outside_the_file_is_skipped() {
        let mut file = ico(&[(16, 32, b"small"), (32, 32, b"large")]);
        // Move the second entry's offset far past the end.
        let offset_field = HEADER_LEN + ENTRY_LEN + 12;
        file[offset_field..offset_field + 4].copy_from_slice(&u32::MAX.to_le_bytes());

        assert_eq!(pick_ico_entry(&file, 32), Some(&b"small"[..]));
    }

    #[test]
    fn the_real_legacy_icon_has_an_entry_for_a_small_icon() {
        let file = include_bytes!("../../../apps/sucrose/assets/TrayIcon.ico");

        let entry = pick_ico_entry(file, 16).expect("the legacy icon has entries");

        assert!(!entry.is_empty());
    }

    #[test]
    fn a_short_tooltip_is_kept_as_is() {
        assert_eq!(truncate_tooltip("Sucrose"), "Sucrose");
    }

    #[test]
    fn a_long_tooltip_is_cut_to_127_units() {
        let text = "a".repeat(300);

        assert_eq!(truncate_tooltip(&text).encode_utf16().count(), MAX_TOOLTIP_UNITS);
    }

    #[test]
    fn the_cut_never_splits_a_surrogate_pair() {
        // 126 ASCII units then an emoji (2 units): 128 in total, so the emoji does not fit.
        let text = format!("{}\u{1F600}", "a".repeat(126));

        let cut = truncate_tooltip(&text);

        assert_eq!(cut, "a".repeat(126));
        assert!(cut.encode_utf16().count() <= MAX_TOOLTIP_UNITS);
    }
}

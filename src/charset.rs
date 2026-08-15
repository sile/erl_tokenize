pub(crate) fn is_atom_head_char(c: char) -> bool {
    matches!(c, 'a'..='z' | 'ß'..='ö' | 'ø'..='ÿ')
}

pub(crate) fn is_atom_non_head_char(c: char) -> bool {
    matches!(
        c,
        'a'..='z' | 'A'..='Z' | '@' | '_' | '0'..='9'
            | 'À'..='Ö'
            | 'Ø'..='Þ'
            | 'ß'..='ö'
            | 'ø'..='ÿ'
    )
}

/// Match erl_scan's effective `?NAMECHAR` set: ASCII alphanumerics,
/// `_`, and `@`. Latin-1 letters are intentionally excluded: erl_scan's
/// macro attempts to include them but chains its Latin-1 clauses with
/// `andalso`, so `ß..ÿ ∩ À..Þ` collapses to the empty set and no
/// Latin-1 letter satisfies the guard in practice.
pub(crate) fn is_namechar(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '@')
}

/// Byte-level fast path for [`is_namechar`]: every namechar is ASCII,
/// so a byte comparison suffices.
pub(crate) fn is_ascii_namechar(b: u8) -> bool {
    matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'@')
}

pub(crate) fn is_variable_head_char(c: char) -> bool {
    // Matches erl_scan: ASCII `A-Z`, `_`, and Latin-1 uppercase letters
    // (`À..Þ` minus the multiplication sign `×`).
    matches!(c, 'A'..='Z' | '_' | 'À'..='Ö' | 'Ø'..='Þ')
}

pub(crate) fn is_variable_non_head_char(c: char) -> bool {
    // Matches erl_scan's `scan_name`: ASCII alphanumerics, `_`, `@`, and
    // Latin-1 letters (`À..Þ` minus `×`, `ß..ÿ` minus `÷`).
    matches!(
        c,
        'a'..='z'
            | 'A'..='Z'
            | '@'
            | '_'
            | '0'..='9'
            | 'À'..='Ö'
            | 'Ø'..='Þ'
            | 'ß'..='ö'
            | 'ø'..='ÿ'
    )
}

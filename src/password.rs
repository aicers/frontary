use passwords::analyzer;

pub(crate) const PASSWORD_MIN_LEN: usize = if cfg!(feature = "cc-password") { 9 } else { 8 };
const PASSWORD_MIN_FORBID_ADJACENT_LEN: usize = 4; // adjacent keyboard characters

#[derive(Clone, Copy, PartialEq)]
pub enum Requirement {
    NoSpace,
    NoControlCharacter,
    MinimumLength(usize), // basic 8, cc 9
    AtLeastOneLowercase,
    AtLeastOneUppercase,
    AtLeastOneDigit,
    AtLeastOneSpecialCharacter,
    NoConsecutiveRepeatingCharacter,           // cc only
    NoMoreThanThreeAdjacentKeyboardCharacters, // cc only
}

#[derive(Clone, Copy, PartialEq)]
pub struct CheckResult {
    pub requirement: Requirement,
    pub passed: bool,
}

#[must_use]
pub fn check_password_requirements(password: &str) -> Vec<CheckResult> {
    let analyzed = analyzer::analyze(password);
    let filtered = analyzed.password();

    let result_basic: Vec<CheckResult> = vec![
        CheckResult {
            requirement: Requirement::NoSpace,
            passed: analyzed.spaces_count() == 0,
        },
        CheckResult {
            requirement: Requirement::NoControlCharacter,
            passed: password == filtered,
        },
        CheckResult {
            requirement: Requirement::MinimumLength(PASSWORD_MIN_LEN),
            passed: analyzed.length() >= PASSWORD_MIN_LEN,
        },
        CheckResult {
            requirement: Requirement::AtLeastOneLowercase,
            passed: analyzed.lowercase_letters_count() > 0,
        },
        CheckResult {
            requirement: Requirement::AtLeastOneUppercase,
            passed: analyzed.uppercase_letters_count() > 0,
        },
        CheckResult {
            requirement: Requirement::AtLeastOneDigit,
            passed: analyzed.numbers_count() > 0,
        },
        CheckResult {
            requirement: Requirement::AtLeastOneSpecialCharacter,
            passed: analyzed.symbols_count() > 0,
        },
    ];
    let result_cc = if cfg!(feature = "cc-password") {
        vec![
            CheckResult {
                requirement: Requirement::NoConsecutiveRepeatingCharacter,
                passed: analyzed.consecutive_count() == 0,
            },
            CheckResult {
                requirement: Requirement::NoMoreThanThreeAdjacentKeyboardCharacters,
                passed: !is_adjacent(password),
            },
        ]
    } else {
        Vec::new()
    };

    [&result_basic[..], &result_cc[..]].concat()
}

const PASSWD_CMP: [&str; 7] = [
    "1234567890",
    "qwertyuiop",
    "QWERTYUIOP",
    "asdfghjkl",
    "ASDFGHJKL",
    "zxcvbnm",
    "ZXCVBNM",
];

pub(crate) fn is_adjacent(password: &str) -> bool {
    for c in PASSWD_CMP {
        for i in 0..=c.len() - PASSWORD_MIN_FORBID_ADJACENT_LEN {
            if let Some(slice) = c.get(i..i + PASSWORD_MIN_FORBID_ADJACENT_LEN) {
                if password.contains(slice) {
                    return true;
                }
            }
        }
        let c_rev: String = c.chars().rev().collect();
        for i in 0..=c_rev.len() - PASSWORD_MIN_FORBID_ADJACENT_LEN {
            if let Some(slice) = c_rev.get(i..i + PASSWORD_MIN_FORBID_ADJACENT_LEN) {
                if password.contains(slice) {
                    return true;
                }
            }
        }
    }
    false
}

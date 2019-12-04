use std::fs;

#[derive(Debug)]
struct Range {
    min: i64,
    max: i64,
}

fn parse_range(filename: &str) -> Range {
    let input = fs::read_to_string(filename).unwrap();
    let mut range = input
        .split("-")
        .map(|x| x.trim().parse::<i64>().unwrap())
        .collect::<Vec<_>>();
    range.sort();
    Range {
        min: range[0],
        max: range[1],
    }
}

fn is_valid_password(password: i64) -> bool {
    let password = password.to_string();
    let mut previous: Option<char> = None;
    let mut has_successive_char = false;
    for c in password.chars() {
        if let Some(p) = previous {
            if c == p {
                has_successive_char = true;
            }
            if c < p {
                return false;
            }
        }
        previous = Some(c);
    }

    has_successive_char
}

fn is_really_valid_password(password: i64) -> bool {
    if !is_valid_password(password) {
        return false;
    }

    let password = password.to_string();
    let mut previous: Option<char> = None;
    let mut successive_char_count = 0;
    for c in password.chars() {
        if let Some(p) = previous {
            if c == p {
                successive_char_count += 1;
            } else {
                if successive_char_count == 1 {
                    return true;
                }
                successive_char_count = 0;
            }
        }
        previous = Some(c);
    }

    successive_char_count == 1
}

fn main() {
    let range = parse_range("input");
    let valid_passwords = (range.min..range.max)
        .filter(|p| is_valid_password(*p))
        .collect::<Vec<_>>();
    let really_valid_passwords = (range.min..range.max)
        .filter(|p| is_really_valid_password(*p))
        .collect::<Vec<_>>();
    println!("4-1:\n{}", valid_passwords.len());
    println!("4-2:\n{}", really_valid_passwords.len());
}

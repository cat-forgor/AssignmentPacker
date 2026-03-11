const MAX_ROWS: usize = 500;
const DEFAULT_COLS: usize = 10000;

pub fn process(text: &str) -> String {
    process_with_cols(text, DEFAULT_COLS)
}

pub fn process_with_cols(text: &str, cols: usize) -> String {
    let cols = cols.max(1);
    let mut screen: Vec<Vec<char>> = vec![Vec::new()];
    let mut row: usize = 0;
    let mut col: usize = 0;
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\x1b' => handle_escape(&mut chars, &mut screen, &mut row, &mut col, cols),
            '\n' => {
                row += 1;
                col = 0;
                ensure_row(&mut screen, row);
            }
            '\r' => {
                col = 0;
            }
            '\x08' => {
                col = col.saturating_sub(1);
            }
            '\t' => {
                let next_tab = (col / 8 + 1) * 8;
                let target = next_tab.min(cols);
                ensure_row(&mut screen, row);
                while col < target {
                    put_char(&mut screen[row], col, ' ');
                    col += 1;
                }
            }
            _ => {
                if row < MAX_ROWS {
                    if col >= cols {
                        row += 1;
                        col = 0;
                    }
                    if row < MAX_ROWS {
                        ensure_row(&mut screen, row);
                        put_char(&mut screen[row], col, c);
                        col += 1;
                    }
                }
            }
        }
    }

    let lines: Vec<String> = screen
        .iter()
        .map(|line| line.iter().collect::<String>().trim_end().to_string())
        .collect();

    let mut result: Vec<String> = Vec::with_capacity(lines.len());
    let mut prev_empty = false;
    for line in lines {
        if line.is_empty() {
            if !prev_empty {
                result.push(String::new());
            }
            prev_empty = true;
        } else {
            result.push(line);
            prev_empty = false;
        }
    }
    while result.first().is_some_and(|l| l.is_empty()) {
        result.remove(0);
    }
    while result.last().is_some_and(|l| l.is_empty()) {
        result.pop();
    }
    result.join("\n")
}

fn ensure_row(screen: &mut Vec<Vec<char>>, row: usize) {
    while screen.len() <= row {
        screen.push(Vec::new());
    }
}

fn put_char(line: &mut Vec<char>, col: usize, ch: char) {
    if col < line.len() {
        line[col] = ch;
    } else {
        while line.len() < col {
            line.push(' ');
        }
        line.push(ch);
    }
}

fn handle_escape(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    screen: &mut Vec<Vec<char>>,
    row: &mut usize,
    col: &mut usize,
    cols: usize,
) {
    match chars.peek() {
        Some('[') => {
            chars.next();
            let (params, cmd) = parse_csi(chars);
            match cmd {
                Some('A') => {
                    let n = params.first().copied().unwrap_or(1).max(1) as usize;
                    *row = row.saturating_sub(n);
                }
                Some('B') => {
                    let n = params.first().copied().unwrap_or(1).max(1) as usize;
                    *row = (*row + n).min(MAX_ROWS - 1);
                    ensure_row(screen, *row);
                }
                Some('C') => {
                    let n = params.first().copied().unwrap_or(1).max(1) as usize;
                    *col = (*col + n).min(cols.saturating_sub(1));
                }
                Some('D') => {
                    let n = params.first().copied().unwrap_or(1).max(1) as usize;
                    *col = col.saturating_sub(n);
                }
                Some('H' | 'f') => {
                    let r = params.first().copied().unwrap_or(1).max(1) as usize - 1;
                    let c = params.get(1).copied().unwrap_or(1).max(1) as usize - 1;
                    *row = r.min(MAX_ROWS - 1);
                    *col = c.min(cols.saturating_sub(1));
                    ensure_row(screen, *row);
                }
                Some('J') => {
                    let mode = params.first().copied().unwrap_or(0);
                    if mode == 2 || mode == 3 {
                        screen.clear();
                        screen.push(Vec::new());
                        *row = 0;
                        *col = 0;
                    }
                }
                Some('K') => {
                    let mode = params.first().copied().unwrap_or(0);
                    if mode == 0 {
                        ensure_row(screen, *row);
                        screen[*row].truncate(*col);
                    }
                }
                _ => {}
            }
        }
        Some(']') => {
            chars.next();
            while let Some(next) = chars.next() {
                if next == '\x07' {
                    break;
                }
                if next == '\x1b' && chars.peek() == Some(&'\\') {
                    chars.next();
                    break;
                }
            }
        }
        Some(&fe) if fe >= '@' && fe <= '_' => {
            chars.next();
        }
        _ => {}
    }
}

fn parse_csi(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> (Vec<u32>, Option<char>) {
    let mut params = Vec::new();
    let mut current: u32 = 0;
    let mut has_digit = false;

    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            chars.next();
            current = current
                .saturating_mul(10)
                .saturating_add(c as u32 - b'0' as u32);
            has_digit = true;
        } else if c == ';' {
            chars.next();
            params.push(current);
            current = 0;
            has_digit = false;
        } else if c.is_ascii_alphabetic() || c == '~' || c == '@' {
            chars.next();
            if has_digit || !params.is_empty() {
                params.push(current);
            }
            return (params, Some(c));
        } else {
            chars.next();
        }
    }
    (params, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_ansi_removes_color_codes() {
        assert_eq!(process("\x1b[31mred\x1b[0m"), "red");
    }

    #[test]
    fn strip_ansi_removes_osc() {
        assert_eq!(process("\x1b]0;title\x07text"), "text");
    }

    #[test]
    fn strip_ansi_preserves_plain_text() {
        assert_eq!(process("hello world"), "hello world");
    }

    #[test]
    fn emulate_cr_overwrites_line() {
        assert_eq!(process("hello\rworld"), "world");
    }

    #[test]
    fn emulate_cr_partial_overwrite() {
        assert_eq!(process("longtext\rhi"), "hingtext");
    }

    #[test]
    fn emulate_backspace() {
        assert_eq!(process("ab\x08c"), "ac");
    }

    #[test]
    fn emulate_backspace_at_start() {
        assert_eq!(process("\x08hello"), "hello");
    }

    #[test]
    fn emulate_crlf_newlines() {
        assert_eq!(process("a\r\nb"), "a\nb");
    }

    #[test]
    fn tab_stops_at_8_columns() {
        assert_eq!(process("a\tb"), "a       b");
        assert_eq!(process("\t"), "");
        assert_eq!(process("\tx"), "        x");
    }

    #[test]
    fn cursor_position_cup() {
        assert_eq!(process("\x1b[2;3Hx"), "  x");
    }

    #[test]
    fn erase_display_2j() {
        assert_eq!(process("hello\x1b[2Jworld"), "world");
    }

    #[test]
    fn erase_line_k() {
        assert_eq!(process("hello\r\x1b[Kworld"), "world");
    }

    #[test]
    fn mixed_ansi_and_text() {
        assert_eq!(process("\x1b[32mgreen\x1b[0m plain"), "green plain");
    }
}

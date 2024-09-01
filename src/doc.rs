use {
    crate::Context,
    std::{fmt::Write, ops},
};

// ? Idea: add a `.doc()` method on builder to store a
// ? Fn pointer to something that impls Display

/*
  Good resource: https://notes.burke.libbey.me/ansi-escape-codes/
*/

/// Inserts the byte 27 into a string. "\e" and "\033"
/// do the same thing
const ESC: &'static str = "\x1b";
/// The default indent size
pub const INDENT_SIZE: u8 = 4;
/// The escape code to reset escape function params
const ESCAPE_SEQ_RESET: &'static str = "\x1b[0m";

#[repr(u8)]
enum EscapeFnParams {
    Reset,
    BoldOrBright,
    Italic,
    Underline = 4,
    BackgroundColorRGB = 8,
    TextColorRGB = 16,
}
impl std::ops::BitOr for EscapeFnParams {
    type Output = u8;

    fn bitor(self, rhs: Self) -> Self::Output {
        self as u8 | rhs as u8
    }
}
#[repr(u8)]

enum Extra {
    None,
    NewlineBeforeChildren,
    NewlineAfterChildren,
    NewlineAfterChildren2x,
}

pub struct DocNodeWithoutSummary {
    blocks: DocNode,
}
impl DocNodeWithoutSummary {
    // ! This depends on the default DocBlocks generated for CLIs
    // ! and could break if the user supplies their own default
    // ! function.
    // ? Maybe a trait could fix the issue?
    pub fn summary(mut self, text: &'static str) -> DocNode {
        self.blocks.children[0].children[0]
            .children
            .last_mut()
            .unwrap()
            .text = text;
        self.blocks
    }
}

#[derive(Debug)]
pub struct DocNode {
    pub text: &'static str,
    pub children: Vec<DocNode>,
    indent: u8,
    column: u8,
    wrapping_sequence: u8,
    wrapper_newlines: u8,
    bg_r: u8,
    bg_g: u8,
    bg_b: u8,
    text_r: u8,
    text_g: u8,
    text_b: u8,
}
impl DocNode {
    // pub const fn id(mut self, text: &'static str) -> Self {
    //     self.id = Some(text);
    //     self
    // }
    pub const fn bold(mut self) -> Self {
        self.wrapping_sequence |= EscapeFnParams::BoldOrBright as u8;
        self
    }
    pub const fn italic(mut self) -> Self {
        self.wrapping_sequence |= EscapeFnParams::Italic as u8;
        self
    }
    pub const fn underline(mut self) -> Self {
        self.wrapping_sequence |= EscapeFnParams::Underline as u8;
        self
    }
    pub const fn indent(mut self) -> Self {
        self.indent += INDENT_SIZE;
        self
    }
    pub const fn bg_color(mut self, red: u8, green: u8, blue: u8) -> Self {
        self.bg_r = red;
        self.bg_g = green;
        self.bg_b = blue;
        self.wrapping_sequence |= EscapeFnParams::BackgroundColorRGB as u8;
        self
    }
    pub const fn text_color(
        mut self,
        red: u8,
        green: u8,
        blue: u8,
    ) -> Self {
        self.text_r = red;
        self.text_g = green;
        self.text_b = blue;
        self.wrapping_sequence |= EscapeFnParams::TextColorRGB as u8;
        self
    }
    pub fn column(mut self, number: u8) -> Self {
        self.column = number;
        self
    }
    pub fn insert_after(
        &mut self,
        name: &str,
        blocks: DocNode,
    ) -> &mut Self {
        match self.children.iter().position(|block| block.text == name) {
            Some(i) => self.children.insert(i + 1, blocks),
            _ => self.children.push(blocks),
        };
        self
    }
    pub fn push(&mut self, blocks: DocNode) {
        self.children.push(blocks);
    }
    // fn find_by_id(&mut self, id: &'static str) -> Option<&mut Self> {
    //     if self.id.is_some_and(|self_id| self_id == id) {
    //         return Some(self);
    //     }
    //     for c in &mut self.children {
    //         let found = c.find_by_id(id);
    //         if found.is_some() {
    //             return found;
    //         }
    //     }
    //     None
    // }
}
impl std::fmt::Display for DocNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.text.is_empty() {
            // Expect top level element to be empty, and to
            // discourage calling `display()` on children
            return Err(std::fmt::Error::default());
        }
        let max_width = 24;

        let mut column_widths = Vec::with_capacity(4);

        // First nesting level, for sections
        for child in self.children.iter() {
            column_widths.clear();
            // Get column widths
            for grandchild in &child.children {
                for (i, great_grandchild) in
                    grandchild.children.iter().enumerate()
                {
                    if i == column_widths.len() {
                        column_widths.push(great_grandchild.text.len())
                    } else if great_grandchild.text.len()
                        > column_widths[i]
                    {
                        column_widths[i] = great_grandchild.text.len();
                    }
                }
            }

            let sum = column_widths.iter().sum::<usize>();
            if let Some(last) = column_widths.last_mut() {
                if sum > max_width {
                    *last -= sum - max_width;
                }
            }

            if !child.text.is_empty() {
                if child.wrapping_sequence == 0 {
                    write!(f, "{}", &child.text)?;
                } else {
                    write_escape_sequence_start(f, &child)?;
                    write!(f, "{}{}", &child.text, ESCAPE_SEQ_RESET)?;
                }
            }
            // if child.wrapper_newlines & Extra::NewlineBeforeChildren as u8
            //     != 0
            // {
            //     writeln!(f)?;
            // }

            // Second nesting level, for block items
            for grandchild in &child.children {
                if grandchild.indent != 0 {
                    write!(
                        f,
                        "{}",
                        " ".repeat(grandchild.indent as usize)
                    )?;
                }
                if !grandchild.text.is_empty() {
                    if grandchild.wrapping_sequence == 0 {
                        write!(f, "{}", grandchild.text)?;
                    } else {
                        write_escape_sequence_start(f, &grandchild)?;
                        write!(
                            f,
                            "{}{}",
                            &grandchild.text, ESCAPE_SEQ_RESET
                        )?;
                    }
                }

                // Third nesting level, for inline items
                wrap_columns(
                    f,
                    &grandchild.children,
                    &column_widths,
                    max_width,
                )?;

                if grandchild.wrapper_newlines
                    == Extra::NewlineAfterChildren as u8
                {
                    writeln!(f)?;
                } /*  else if grandchild.wrapper_newlines
                      == Extra::NewlineAfterChildren2x as u8
                  {
                      writeln!(f, "\n")?;
                  } */
            }
            if child.wrapper_newlines == Extra::NewlineAfterChildren as u8
            {
                writeln!(f)?;
            } /*  else if child.wrapper_newlines
                  == Extra::NewlineAfterChildren2x as u8
              {
                  writeln!(f, "\n")?;
              } */
        }
        Ok(())
    }
}

fn word_bounds(text: &str, index: usize) -> ops::Range<usize> {
    let mut r = index..index;
    let bytes = text.as_bytes();
    while r.start > 0 {
        r.start -= 1;
        match bytes[r.start] {
            b' ' | b'\t' | b'\n' | b'\r' => {
                r.start += 1;
                break;
            }
            _ => (),
        }
    }
    while r.end < text.len() {
        match bytes[r.end] {
            b' ' | b'\t' | b'\n' | b'\r' => break,
            _ => r.end += 1,
        }
    }
    r
}

fn write_escape_sequence_start(
    f: &mut impl Write,
    block: &DocNode,
) -> std::fmt::Result {
    write!(f, "\x1b[")?;
    let mut found_one = false;
    if block.wrapping_sequence & EscapeFnParams::BoldOrBright as u8 != 0 {
        write!(f, "1")?;
        found_one = true;
    }
    if block.wrapping_sequence & EscapeFnParams::Italic as u8 != 0 {
        if found_one {
            write!(f, ";")?;
        } else {
            found_one = true
        }
        write!(f, "3")?;
    }
    if block.wrapping_sequence & EscapeFnParams::Underline as u8 != 0 {
        if found_one {
            write!(f, ";")?;
        } else {
            found_one = true
        }
        write!(f, "4")?;
    }
    if block.wrapping_sequence & EscapeFnParams::TextColorRGB as u8 != 0 {
        if found_one {
            write!(f, ";")?;
        } else {
            found_one = true
        }
        write!(
            f,
            "38;2;{};{};{}",
            block.text_r, block.text_g, block.text_b
        )?;
    }
    if block.wrapping_sequence & EscapeFnParams::BackgroundColorRGB as u8
        != 0
    {
        if found_one {
            write!(f, ";")?;
        }
        write!(f, "48;2;{};{};{}", block.bg_r, block.bg_g, block.bg_b)?;
    }

    write!(f, "m")
}

// struct Block<'a, F>
// where
//     F: Fn(usize) -> String,
// {
//     a: String,
//     b: &'static str,
//     c: crate::Action,
//     d: fn(usize) -> usize,
//     f: Box<dyn Fn(usize) -> usize>,
//     g: &'a dyn Fn(usize) -> usize,
//     e: F,
// }
//
// struct EscapeCode {
//   f: char,
//   params: EscapeFnParams,
// }
// impl EscapeCode {
//   pub fn func()
// }
// pub fn color(r: u8, g: u8, b: u8) -> (String, u8) {
//     "\\033["
// }

pub const fn block(children: Vec<DocNode>) -> DocNode {
    DocNode {
        text: "",
        children,
        indent: INDENT_SIZE,
        column: 0,
        wrapping_sequence: 0,
        wrapper_newlines: Extra::NewlineAfterChildren as u8,
        bg_r: 0,
        bg_g: 0,
        bg_b: 0,
        text_r: 0,
        text_g: 0,
        text_b: 0,
    }
}
pub const fn inline(children: Vec<DocNode>) -> DocNode {
    DocNode {
        text: "",
        children,
        indent: 0,
        column: 0,
        wrapping_sequence: 0,
        wrapper_newlines: 0,
        bg_r: 0,
        bg_g: 0,
        bg_b: 0,
        text_r: 0,
        text_g: 0,
        text_b: 0,
    }
}
pub const fn text(text: &'static str) -> DocNode {
    DocNode {
        text,
        children: Vec::new(),
        indent: 0,
        column: 0,
        wrapping_sequence: 0,
        wrapper_newlines: 0,
        bg_r: 0,
        bg_g: 0,
        bg_b: 0,
        text_r: 0,
        text_g: 0,
        text_b: 0,
    }
}

pub fn empty_doc(_: &Context, blocks: DocNodeWithoutSummary) -> DocNode {
    blocks.summary("")
}

pub fn cli_doc(c: &Context) -> String {
    let mut s = String::with_capacity(0);
    let spacing = "    ";

    s.push_str("\n\nSYNOPSIS\n");
    s.push_str(spacing);
    s.push_str(
        c.router.names
            [c.router.segments[c.selected as usize].name as usize],
    );
    if !c.router.summaries[c.selected as usize + c.router.options.len()]
        .is_empty()
    {
        s.push_str(" - ");
        s.push_str(
            c.router.summaries
                [c.selected as usize + c.router.options.len()],
        );
    }
    // TODO: Change this to per-command options
    if !c.router.options.is_empty() {
        s.push_str(" [options...]");
    }
    if c.router.tree[c.selected as usize].child_span > 0 {
        s.push_str(" [command]");
    } else {
        match c.router.segments[c.selected as usize].operands {
            0 => (),
            1 => {
                s.push_str(" <operand>");
            }
            u16::MAX => s.push_str(" <operands...>"),
            n => {
                s.push_str(" <operands; ");
                s.push_str(&n.to_string());
                s.push('>');
            }
        }
    }
    // TODO: Change this to per-command options
    if !c.router.options.is_empty() {
        s.push_str("\n\nOPTIONS");
        for (i, opt) in c.router.options.iter().enumerate() {
            s.push('\n');
            s.push_str(spacing);
            s.push_str("--");
            s.push_str(c.router.names[opt.name as usize]);
            if let Some((_, c)) = c
                .router
                .short_option_mappers
                .iter()
                .find(|(opt_index, _)| *opt_index == i as u16)
            {
                s.push_str(", -");
                s.push(*c);
            }
            s.push_str("\t\t");
            if !c.router.summaries[i].is_empty() {
                s.push_str(c.router.summaries[i]);
            }
        }
    }
    if c.router.tree[c.selected as usize].child_span > 0 {
        let mut child_index = c.selected + 1;
        s.push_str("\n\nCOMMANDS");
        while child_index
            < c.selected
                + c.router.tree[c.selected as usize].child_span
                + 1
        {
            s.push('\n');
            s.push_str(spacing);
            s.push_str(
                c.router.names[c.router.segments[child_index as usize].name
                    as usize],
            );
            if !c.router.summaries
                [child_index as usize + c.router.options.len()]
            .is_empty()
            {
                s.push('\n');
                s.push_str(spacing);
                s.push_str("  ");
                s.push_str(
                    c.router.summaries
                        [child_index as usize + c.router.options.len()],
                );
            }
            s.push('\n');
            // Skip to next sibling segment
            child_index +=
                c.router.tree[child_index as usize].child_span + 1;
        }
    }
    s
}
pub fn default_doc_blocks(c: &Context) -> DocNodeWithoutSummary {
    let mut blocks = DocNodeWithoutSummary {
        blocks: DocNode {
            text: "",
            children: vec![
                block(vec![inline(vec![
                    // text(
                    //     c.router.names[c.router.segments
                    //         [c.selected as usize]
                    //         .name
                    //         as usize],
                    // ),
                    // text(" - "),
                    text("").column(3),
                ])]),
                // block(vec![inline(vec![
                //     text("Usage:  "),
                //     text("command example A").column(2),
                //     text("command example B").column(2),
                // ])]),
                block(vec![text("Options").bold()]),
            ],
            indent: 0,
            column: 0,
            wrapping_sequence: 0,
            wrapper_newlines: 0,
            bg_r: 0,
            bg_g: 0,
            bg_b: 0,
            text_r: 0,
            text_g: 0,
            text_b: 0,
        },
    };

    // * Print commands and their summaries
    // TODO: Maybe the DocNodes can be restructured so that
    //       the summary is always the first node, so the
    //       doc fn can return an iterator and each summary
    //       can be got by just calling next() once. The
    //       summary could also be stored as the top level
    //       DocNode's text, being a special case.
    //       Alternatively, have get_summary() fn that must
    //       be implemented when the user supplies a custom
    //       doc() fn, which would advance the iterator as
    //       needed until the summary is found.
    if c.router.tree[c.selected as usize].child_span > 0 {
        let mut child_index = c.selected + 1;
        let mut cmds = vec![];
        while child_index
            < c.selected
                + c.router.tree[c.selected as usize].child_span
                + 1
        {
            cmds.push(
                inline(vec![
                    text(
                        c.router.names[c.router.segments
                            [child_index as usize]
                            .name
                            as usize],
                    ),
                    text("dwaoidmwaodin oawd noawnd dwai dowan odnaownaodiwado"),
                ])
                .indent(),
            );
            // Skip to next sibling segment
            child_index +=
                c.router.tree[child_index as usize].child_span + 1;
        }
        blocks.blocks.push(block(vec![text("Commands").bold()]));
        blocks.blocks.push(block(cmds));
    }

    blocks
}
fn wrap_columns(
    f: &mut impl Write,
    columns: &[DocNode],
    column_widths: &[usize],
    max_len: usize,
) -> std::fmt::Result {
    // Represents filled character slots which acrue as text from
    // that column is written. the column_length - cursor is the
    // amount on the the next column should prepend to its text
    let mut cursors = vec![0; columns.len()];
    let mut col = 0;
    let mut chars_to_write =
        columns.iter().fold(0, |acc, block| acc + block.text.len());

    while chars_to_write > 0 {
        // Remaining length <= column width
        if cursors[col] == columns[col].text.len() {
            write!(f, "{}", " ".repeat(column_widths[col]))?;
        } else if columns[col].text.len() - cursors[col]
            <= column_widths[col]
        {
            if columns[col].wrapping_sequence != 0 {
                write_escape_sequence_start(f, &columns[col])?;
            }

            let text =
                &columns[col].text[cursors[col]..columns[col].text.len()];
            cursors[col] = columns[col].text.len();
            write!(f, "{}", text)?;
            chars_to_write -= text.len();
            if col != columns.len() - 1
                && column_widths[col] - text.len() > 0
            {
                write!(
                    f,
                    "{}",
                    " ".repeat(column_widths[col] - text.len())
                )?;
            }

            if columns[col].wrapping_sequence != 0 {
                write!(f, "{}", ESCAPE_SEQ_RESET)?;
            }
        } else {
            // Greater than col width, break
            if columns[col].wrapping_sequence != 0 {
                write_escape_sequence_start(f, &columns[col])?;
            }

            let text = &columns[col].text[cursors[col]..];
            let bounds = word_bounds(text, column_widths[col]);
            // cursors[col] += column_widths[col];
            // write!(f, "{}", text)?;
            // chars_to_write -= text.len();

            // let text = columns[col].text;
            if column_widths[col] == bounds.start
                || column_widths[col] == bounds.start + 1
            {
                // write line up to start
                write!(f, "{}", &text[0..bounds.start])?;
                chars_to_write -= bounds.start;
                cursors[col] += bounds.start;
            } else if column_widths[col] == bounds.end {
                // write line up to end
                write!(f, "{}", &text[0..bounds.end])?;
                chars_to_write -= bounds.end;
                cursors[col] += bounds.end;
            } else {
                // break word
                write!(f, "{}-", &text[0..column_widths[col] - 1])?;
                chars_to_write -= column_widths[col] - 1;
                cursors[col] += column_widths[col] - 1;

                // let mut i = max_len;
                // write!(f, "{}", &text[0..i])?;
                // while i < text.len() {
                //     let max = if text.len() - i < max_len {
                //         text.len()
                //     } else {
                //         i + max_len
                //     };
                //     write!(f, "{}", &text[i..max])?;
                //     i = max
                // }
            }

            if columns[col].wrapping_sequence != 0 {
                write!(f, "{}", ESCAPE_SEQ_RESET)?;
            }
        };

        if col == cursors.len() - 1 {
            col = 0;
            writeln!(f)?;
        } else {
            col += 1;
        }
    }
    Ok(())
}

fn single_bit(bitset: u8) -> bool {
    bitset != 0 && (bitset & (bitset - 1)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_check_if_only_a_single_bitset_bit_is_set() {
        assert_eq!(single_bit(EscapeFnParams::Italic as u8), true);
        assert_eq!(single_bit(EscapeFnParams::BoldOrBright as u8), true);
        assert_eq!(
            single_bit(
                EscapeFnParams::BoldOrBright as u8
                    | EscapeFnParams::Italic as u8
            ),
            false
        );
        assert_eq!(single_bit(EscapeFnParams::Underline as u8), true);
        assert_eq!(
            single_bit(
                EscapeFnParams::Italic as u8
                    | EscapeFnParams::Underline as u8
            ),
            false
        );
    }
    #[test]
    fn should_wrap_columns() {
        let mut blocks = [
            DocNode {
                text: "Row1:",
                children: Vec::new(),
                indent: 0,
                column: 0,
                wrapping_sequence: 0,
                wrapper_newlines: 0,
                bg_r: 0,
                bg_g: 0,
                bg_b: 0,
                text_r: 0,
                text_g: 0,
                text_b: 0,
            },
            DocNode {
                text: " Some long tx",
                children: Vec::new(),
                indent: 0,
                column: 0,
                wrapping_sequence: 0,
                wrapper_newlines: 0,
                bg_r: 0,
                bg_g: 0,
                bg_b: 0,
                text_r: 0,
                text_g: 0,
                text_b: 0,
            },
            DocNode {
                text: " Some extra super long text",
                children: Vec::new(),
                indent: 0,
                column: 0,
                wrapping_sequence: 0,
                wrapper_newlines: 0,
                bg_r: 0,
                bg_g: 0,
                bg_b: 0,
                text_r: 0,
                text_g: 0,
                text_b: 0,
            },
        ];
        let mut s = String::new();
        wrap_columns(&mut s, &blocks, &[5, 13, 11], 30).unwrap();
        assert_eq!(
            [
                "Row1: Some long tx Some extra",
                "                   super long",
                "                   text"
            ]
            .join("\n"),
            s
        );

        s.clear();
        wrap_columns(&mut s, &blocks, &[5, 9, 26], 40).unwrap();
        assert_eq!(
            [
                "Row1: Some long Some extra super long",
                "      tx        text",
            ]
            .join("\n"),
            s
        );

        s.clear();
        blocks[0].wrapping_sequence |= EscapeFnParams::BoldOrBright as u8;
        blocks[1].wrapping_sequence |= EscapeFnParams::Italic as u8;
        blocks[2].wrapping_sequence |=
            EscapeFnParams::BackgroundColorRGB as u8;
        blocks[2].bg_r = 128;
        blocks[2].bg_g = 128;
        blocks[2].bg_b = 128;
        wrap_columns(&mut s, &blocks, &[6, 9, 26], 40).unwrap();
        assert_eq!(
            [
                "\x1b[1mRow1:\x1b[0m\x1b[3m Some long\x1b[0m\x1b[48;2;128;128;128m Some extra super long\x1b[0m",
                "      \x1b[3mtx\x1b[0m        \x1b[48;2;128;128;128mtext\x1b[0m",
            ].join("\n"),
            s
        );

        let blocks = [
            DocNode {
                text: "Row1:",
                children: Vec::new(),
                indent: 0,
                column: 0,
                wrapping_sequence: 0,
                wrapper_newlines: 0,
                bg_r: 0,
                bg_g: 0,
                bg_b: 0,
                text_r: 0,
                text_g: 0,
                text_b: 0,
            },
            DocNode {
                text: " Some long tx",
                children: Vec::new(),
                indent: 0,
                column: 0,
                wrapping_sequence: 0,
                wrapper_newlines: 0,
                bg_r: 0,
                bg_g: 0,
                bg_b: 0,
                text_r: 0,
                text_g: 0,
                text_b: 0,
            },
            DocNode {
                text: " Some extra super long text",
                children: Vec::new(),
                indent: 0,
                column: 0,
                wrapping_sequence: 0,
                wrapper_newlines: 0,
                bg_r: 0,
                bg_g: 0,
                bg_b: 0,
                text_r: 0,
                text_g: 0,
                text_b: 0,
            },
        ];
        let mut s = String::new();
        wrap_columns(&mut s, &blocks, &[10, 13, 11], 30).unwrap();
        assert_eq!(
            [
                "Row1:      Some long tx Some extra",
                "                        super long",
                "                        text"
            ]
            .join("\n"),
            s
        );

        let blocks = [
            DocNode {
                text: "add",
                children: Vec::new(),
                indent: 0,
                column: 0,
                wrapping_sequence: 0,
                wrapper_newlines: 0,
                bg_r: 0,
                bg_g: 0,
                bg_b: 0,
                text_r: 0,
                text_g: 0,
                text_b: 0,
            },
            DocNode {
                text:
                    " dwaoidmwaodin oawd noawnd dwai dowan odnaownaodiwado",
                children: Vec::new(),
                indent: 0,
                column: 0,
                wrapping_sequence: 0,
                wrapper_newlines: 0,
                bg_r: 0,
                bg_g: 0,
                bg_b: 0,
                text_r: 0,
                text_g: 0,
                text_b: 0,
            },
        ];
        let mut s = String::new();
        wrap_columns(&mut s, &blocks, &[3, 19], 30).unwrap();

        assert_eq!(
            [
                "add dwaoidmwaodin oawd",
                "    noawnd dwai dowan",
                "    odnaownaodiwado",
            ]
            .join("\n"),
            s
        );
    }
    #[test]
    fn should_find_word_bounds() {
        assert_eq!(word_bounds("text", 2), 0..4);
        assert_eq!(word_bounds("some text", 6), 5..9);
        assert_eq!(word_bounds("some text", 5), 5..9);
        // ? Should it not count when the index is at the end?
        assert_eq!(word_bounds("some text", 9), 5..9);
    }
    #[test]
    fn should_build_docblocks_from_builders() {
        let expected_blocks = DocNode {
            text: "",
            children: vec![DocNode {
                text: "",
                children: vec![
                    DocNode {
                        text: "Bold, italic, and color",
                        children: Vec::new(),
                        indent: 0,
                        column: 0,
                        wrapping_sequence: EscapeFnParams::BoldOrBright
                            as u8
                            | EscapeFnParams::Italic as u8
                            | EscapeFnParams::TextColorRGB as u8,
                        wrapper_newlines: Extra::NewlineAfterChildren
                            as u8,
                        bg_r: 0,
                        bg_g: 0,
                        bg_b: 0,
                        text_r: 128,
                        text_g: 18,
                        text_b: 8,
                    },
                    DocNode {
                        text: "Some text",
                        children: Vec::new(),
                        indent: 0,
                        column: 0,
                        wrapping_sequence: 0,
                        wrapper_newlines: 0,
                        bg_r: 0,
                        bg_g: 0,
                        bg_b: 0,
                        text_r: 0,
                        text_g: 0,
                        text_b: 0,
                    },
                    DocNode {
                        text: " with ",
                        children: Vec::new(),
                        indent: 0,
                        column: 0,
                        wrapping_sequence: 0,
                        wrapper_newlines: 0,
                        bg_r: 0,
                        bg_g: 0,
                        bg_b: 0,
                        text_r: 0,
                        text_g: 0,
                        text_b: 0,
                    },
                    DocNode {
                        text: "underline",
                        children: Vec::new(),
                        indent: 0,
                        column: 0,
                        wrapping_sequence: EscapeFnParams::Underline as u8,
                        wrapper_newlines: 0,
                        bg_r: 0,
                        bg_g: 0,
                        bg_b: 0,
                        text_r: 0,
                        text_g: 0,
                        text_b: 0,
                    },
                    DocNode {
                        text: " in it.",
                        children: Vec::new(),
                        indent: 0,
                        column: 0,
                        wrapping_sequence: 0,
                        wrapper_newlines: 0,
                        bg_r: 0,
                        bg_g: 0,
                        bg_b: 0,
                        text_r: 0,
                        text_g: 0,
                        text_b: 0,
                    },
                ],
                indent: INDENT_SIZE,
                column: 0,
                wrapping_sequence: 0,
                wrapper_newlines: Extra::NewlineAfterChildren as u8,
                bg_r: 0,
                bg_g: 0,
                bg_b: 0,
                text_r: 0,
                text_g: 0,
                text_b: 0,
            }],
            bg_r: 0,
            bg_g: 0,
            bg_b: 0,
            text_r: 0,
            text_g: 0,
            text_b: 0,
            indent: 0,
            column: 0,
            wrapper_newlines: 0,
            wrapping_sequence: 0,
        };

        let actual_blocks = block(vec![block(vec![
            text("Bold, italic, and color")
                .bold()
                .italic()
                .text_color(128, 18, 8),
            text("Some text"),
            text(" with "),
            text("underline").underline(),
            text(" in it."),
        ])]);
        assert_eq!(
            expected_blocks.children[0].text,
            actual_blocks.children[0].text
        );
        assert_eq!(
            expected_blocks.children[0].text_r,
            actual_blocks.children[0].text_r
        );
        assert_eq!(
            expected_blocks.children[0].text_g,
            actual_blocks.children[0].text_g
        );
        assert_eq!(
            expected_blocks.children[0].text_b,
            actual_blocks.children[0].text_b
        );
        assert_eq!(
            expected_blocks.children[0].wrapping_sequence,
            actual_blocks.children[0].wrapping_sequence
        );
        assert_eq!(
            expected_blocks.children[0].wrapper_newlines,
            actual_blocks.children[0].wrapper_newlines
        );

        assert_eq!(
            expected_blocks.children[0].children.len(),
            actual_blocks.children[0].children.len()
        );
        assert_eq!(
            expected_blocks.children[0].children[0].text,
            actual_blocks.children[0].children[0].text
        );
        assert_eq!(
            expected_blocks.children[0].children[1].text,
            actual_blocks.children[0].children[1].text
        );
        assert_eq!(
            expected_blocks.children[0].children[2].text,
            actual_blocks.children[0].children[2].text
        );
        assert_eq!(
            expected_blocks.children[0].children[2].wrapping_sequence,
            actual_blocks.children[0].children[2].wrapping_sequence
        );
    }
    #[test]
    fn should_restrict_elements_to_their_specified_columns() {
        //         let actual = block(vec![block(vec![inline(vec![
        //             text("Usage: "),
        //             text("command example A").column(2),
        //             text("command example B").column(2),
        //         ])])])
        //         .to_string();
        //         let expected = "Usage: command example A
        //        command example B
        // ";
        //         assert_eq!(expected, actual);

        let actual = block(vec![block(vec![inline(vec![
            text("\nUsage: "),
            text("command example A").column(2),
            text("command example B").column(2),
            text("command overflowing example C").column(2),
        ])])])
        .to_string();
        let expected = "
Usage: command example A
       command example B
       command
       overflowing
       example C
";
        assert_eq!(expected, actual);
    }
    #[test]
    fn should_generate_a_formatted_string_from_blocks() {
        let blocks = DocNode {
            text: "",
            children: vec![DocNode {
                text: "",
                children: vec![
                    DocNode {
                        text: "Bold, italic, and color",
                        children: Vec::new(),
                        indent: 0,
                        column: 0,
                        wrapping_sequence: EscapeFnParams::BoldOrBright
                            as u8
                            | EscapeFnParams::Italic as u8
                            | EscapeFnParams::TextColorRGB as u8,
                        wrapper_newlines: Extra::NewlineAfterChildren
                            as u8,
                        bg_r: 0,
                        bg_g: 0,
                        bg_b: 0,
                        text_r: 128,
                        text_g: 18,
                        text_b: 8,
                    },
                    DocNode {
                        text: "Some text",
                        children: Vec::new(),
                        indent: 0,
                        column: 0,
                        wrapping_sequence: 0,
                        wrapper_newlines: 0,
                        bg_r: 0,
                        bg_g: 0,
                        bg_b: 0,
                        text_r: 0,
                        text_g: 0,
                        text_b: 0,
                    },
                    DocNode {
                        text: " with ",
                        children: Vec::new(),
                        indent: 0,
                        column: 0,
                        wrapping_sequence: 0,
                        wrapper_newlines: 0,
                        bg_r: 0,
                        bg_g: 0,
                        bg_b: 0,
                        text_r: 0,
                        text_g: 0,
                        text_b: 0,
                    },
                    DocNode {
                        text: "underline",
                        children: Vec::new(),
                        indent: 0,
                        column: 0,
                        wrapping_sequence: EscapeFnParams::Underline as u8,
                        wrapper_newlines: 0,
                        bg_r: 0,
                        bg_g: 0,
                        bg_b: 0,
                        text_r: 0,
                        text_g: 0,
                        text_b: 0,
                    },
                    DocNode {
                        text: " in it.",
                        children: Vec::new(),
                        indent: 0,
                        column: 0,
                        wrapping_sequence: 0,
                        wrapper_newlines: 0,
                        bg_r: 0,
                        bg_g: 0,
                        bg_b: 0,
                        text_r: 0,
                        text_g: 0,
                        text_b: 0,
                    },
                ],
                indent: INDENT_SIZE,
                column: 0,
                wrapping_sequence: 0,
                wrapper_newlines: Extra::NewlineAfterChildren as u8,
                bg_r: 0,
                bg_g: 0,
                bg_b: 0,
                text_r: 0,
                text_g: 0,
                text_b: 0,
            }],
            bg_r: 0,
            bg_g: 0,
            bg_b: 0,
            text_r: 0,
            text_g: 0,
            text_b: 0,
            indent: 0,
            column: 0,
            wrapper_newlines: 0,
            wrapping_sequence: 0,
        };
        assert_eq!(
            blocks.to_string(),
            "\x1b[1;3;38;2;128;18;8mBold, italic, and color\x1b[0m
Some text with \x1b[4munderline\x1b[0m in it.
"
        );

        let b = DocNode {
            text: "",
            children: vec![block(vec![text("Options\n").bold()])],
            indent: 0,
            column: 0,
            wrapping_sequence: 0,
            wrapper_newlines: 0,
            bg_r: 0,
            bg_g: 0,
            bg_b: 0,
            text_r: 0,
            text_g: 0,
            text_b: 0,
        };
        assert_eq!("\x1b[1mOptions\n\x1b[0m\n", b.to_string());

        let b = DocNode {
            text: "",
            children: vec![block(vec![text("Options\n").bold().italic()])],
            indent: 0,
            column: 0,
            wrapping_sequence: 0,
            wrapper_newlines: 0,
            bg_r: 0,
            bg_g: 0,
            bg_b: 0,
            text_r: 0,
            text_g: 0,
            text_b: 0,
        };
        assert_eq!("\x1b[1;3mOptions\n\x1b[0m\n", b.to_string());

        let b = DocNode {
            text: "",
            children: vec![block(vec![text("Options\n")
                .text_color(128, 0, 70)
                .bg_color(128, 128, 128)])],
            indent: 0,
            column: 0,
            wrapping_sequence: 0,
            wrapper_newlines: 0,
            bg_r: 0,
            bg_g: 0,
            bg_b: 0,
            text_r: 0,
            text_g: 0,
            text_b: 0,
        };
        assert_eq!(
            "\x1b[38;2;128;0;70;48;2;128;128;128mOptions\n\x1b[0m\n",
            b.to_string()
        );

        let b = DocNodeWithoutSummary {
            blocks: DocNode {
                text: "",
                children: vec![
                    block(vec![inline(vec![
                        text("program"),
                        text(" - "),
                        text("").column(3),
                    ])]),
                    block(vec![text("Options\n").bold()]),
                ],
                indent: 0,
                column: 0,
                wrapping_sequence: 0,
                wrapper_newlines: 0,
                bg_r: 0,
                bg_g: 0,
                bg_b: 0,
                text_r: 0,
                text_g: 0,
                text_b: 0,
            },
        }
        .summary("summ");
        assert_eq!(
            "program - summ

\x1b[1mOptions\n\x1b[0m
",
            b.to_string()
        );
    }
    #[test]
    fn should_insert_a_block_after_a_named_block() {
        let mut blocks = DocNode {
            text: "",
            children: vec![
                DocNode {
                    text: "a",
                    children: Vec::new(),
                    text_r: 0,
                    text_g: 0,
                    text_b: 0,
                    bg_r: 0,
                    bg_g: 0,
                    bg_b: 0,
                    indent: 0,
                    column: 0,
                    wrapping_sequence: 0,
                    wrapper_newlines: 0,
                },
                DocNode {
                    text: "b",
                    children: Vec::new(),
                    text_r: 0,
                    text_g: 0,
                    text_b: 0,
                    bg_r: 0,
                    bg_g: 0,
                    bg_b: 0,
                    indent: 0,
                    column: 0,
                    wrapping_sequence: 0,
                    wrapper_newlines: 0,
                },
                DocNode {
                    text: "c",
                    children: Vec::new(),
                    text_r: 0,
                    text_g: 0,
                    text_b: 0,
                    bg_r: 0,
                    bg_g: 0,
                    bg_b: 0,
                    indent: 0,
                    column: 0,
                    wrapping_sequence: 0,
                    wrapper_newlines: 0,
                },
            ],
            text_r: 0,
            text_g: 0,
            text_b: 0,
            bg_r: 0,
            bg_g: 0,
            bg_b: 0,
            indent: 0,
            column: 0,
            wrapping_sequence: 0,
            wrapper_newlines: 0,
        };

        blocks.insert_after(
            "b",
            DocNode {
                text: "d",
                children: Vec::new(),
                text_r: 0,
                text_g: 0,
                text_b: 0,
                bg_r: 0,
                bg_g: 0,
                bg_b: 0,
                indent: 0,
                column: 0,
                wrapping_sequence: 0,
                wrapper_newlines: 0,
            },
        );
        assert_eq!("d", blocks.children[2].text);
    }
}

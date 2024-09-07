//! # Features
//! * **eq-separator** -
//!   Allow separating options from option-arguments
//!   with a '='
//! * **single-hyphen-option-names** -
//!   Changes options to expect a single "-" prefix
//!   instead of "--", and short options are disabled

mod builder;
mod doc;
// mod uri;
use std::{ffi::OsString, io, ops::Range, str::FromStr};
pub use {builder::*, doc::*, opt_map::optmap};

pub type Action = fn(c: Context) -> io::Result<()>;
pub type DocGen =
    fn(c: &Context, doc: doc::DocNodeWithoutSummary) -> DocNode;

pub struct Arg<'a> {
    context: &'a Context<'a>,
    range: Range<u16>,
}
impl<'a> Arg<'a> {
    // Decision:
    // The `Result` wraps the `Option` instead of the other way
    // around so that a default can be assigned. If it were the
    // other way around, the default would come before the type
    // conversion and would need to be given in string form. The
    // other way around would be better if those issues went away
    //
    /// Like `Iterator::next()`, except it attempts to convert from
    /// the `OsString` argument to `T`. It doesn't use the name
    /// `next` because for single-value options it sounds more
    /// natural.
    pub fn value<T: FromStr>(&mut self) -> io::Result<Option<T>> {
        if self.range.is_empty() {
            return Ok(None);
        }
        self.range.start += 1;

        match self.context.saved_args[self.range.start as usize - 1]
            .to_str()
        {
            None => Err(io::Error::new(io::ErrorKind::InvalidData, "")),
            Some(a) => match a.parse::<T>() {
                Ok(v) => Ok(Some(v)),
                _ => Err(io::Error::new(io::ErrorKind::InvalidData, "")),
            },
        }
    }
}

pub struct Context<'a> {
    /// The selected `Segment`'s operands followed by any
    /// arguments found after a terminator. `operands_end`
    /// marks the start of any arguments after the terminator
    operands: Vec<OsString>,
    /// Option arguments
    pub saved_args: Vec<OsString>,
    // Index to options and index to either saved_args
    // or arg_ranges if the option kind is `Multiple`
    pub option_args: Vec<(u16, u16)>,
    // For options that expect multiple args.
    // When there are multiple args for an option, they
    // should appear next to each other in `saved_args`
    pub arg_ranges: Vec<Range<u16>>,
    // How many times the option was found
    pub option_occurrences: Vec<u8>,
    pub router: &'a Router,
    pub selected: u16,
    /// Where operands end and args after a terminator begin
    operands_end: u16,
    path_params: u8,
}
impl<'a> Context<'a> {
    #[inline]
    pub fn operands(&self) -> &[OsString] {
        &self.operands
            [self.path_params as usize..self.operands_end as usize]
    }
    /// Return an iterater-like to get an option's value(s)
    pub fn opt(&self, option: impl Into<usize> + Copy) -> Arg {
        let mut arg = Arg {
            context: self,
            range: 0..0,
        };
        if self.option_occurrences[option.into()] == 0 {
            return arg;
        }
        if let OptArgKind::KeyOnly =
            self.router.options[option.into()].kind
        {
            return arg;
        }
        arg.range.start = self
            .option_args
            .iter()
            .find(|(o, _)| *o as usize == option.into())
            // Unwrap should be safe because non-KeyOnly options have values,
            // and their occurrence was already checked
            .unwrap()
            .1;
        match self.router.options[option.into()].kind {
            OptArgKind::Multiple => {
                arg.range.end =
                    self.arg_ranges[arg.range.start as usize].end;
                arg.range.start =
                    self.arg_ranges[arg.range.start as usize].start;
            }
            _ => {
                arg.range.end = arg.range.start + 1;
            }
        };

        arg
    }
    #[inline]
    pub fn path_params(&self) -> &[OsString] {
        &self.operands[..self.path_params as usize]
    }
    #[inline]
    pub fn terminated_args(&self) -> &[OsString] {
        &self.operands[self.operands_end as usize..]
    }
}

/// Holds data necessary to map a parsed argument to an option
#[derive(Debug)]
pub struct Opt {
    /// An index into the shared list of names
    pub name: u16,
    pub kind: OptArgKind,
}

/// Used during parsing to determine if it needs to be cached
#[derive(Debug)]
pub enum OptArgKind {
    /// The option has no option-argument
    KeyOnly,
    /// Expects a single option-argument, and an
    /// occurrence of the option overrides any previous
    /// occurrence of the same option
    Single,
    /// Expects an option-argument, and an occurrence
    /// of the option adds to a list
    Multiple,
}

#[derive(Clone, Copy)]
pub enum OptGroupRules {
    AnyOf,
    OneOf,
    Required,
}

pub struct Router {
    // Decision:
    // Couldn't use a const `TreePack` because `SIZE` in
    // `const r: Router<SIZE> = router!(O, C);` had to
    // be known by the user
    pub tree: &'static [TreeNode],
    pub segments: &'static [Segment],
    pub actions: &'static [Action],
    // Bitmask: exclusive, required, and cascades bools
    // The u8s act as `OptGroupRules`, but are stored
    // as u8s to avoid casting at runtime
    pub opt_group_rules: &'static [u8],
    // List of all commands' groups; the commands themselves
    // hold ranges into this
    pub opt_groups: &'static [&'static [u16]],
    pub options: &'static [Opt],
    pub short_option_mappers: &'static [(u16, char)],
    pub names: &'static [&'static str],
    pub help_opt_index: Option<u16>,
}
impl Router {
    #[inline(always)]
    pub fn parse(
        &self,
        args: impl IntoIterator<Item = OsString>,
    ) -> io::Result<Context> {
        parse_cli_route(self, args)
    }
    #[inline(always)]
    pub fn context(&self) -> io::Result<Context> {
        parse_cli_route(self, std::env::args_os().skip(1))
    }
    pub fn run(&self) -> io::Result<()> {
        let c = parse_cli_route(self, std::env::args_os().skip(1))?;
        self.actions[c.selected as usize](c)
    }
}

/// Things needed at runtime for the segment (summary stored separately)
#[derive(Debug, Clone, Copy)]
pub struct Segment {
    operands: u16,
    /// First 4 bits specify a length of groups as an offset,
    /// from the index. The remaining 12 bits are for the
    /// index.
    /// This means a `Segment` can have up to 15 option
    /// groups, and the total number of groups for the
    /// `Router` cannot exceed 8,190.
    ///
    /// Will be 0 if it has no groups
    pub opt_groups: u16,
    /// An index into the shared list of names
    pub name: u16,
}

#[derive(Clone, Copy)]
pub struct TreeNode {
    child_span: u16,
    parent: u16,
}

fn add_found_option(
    index: usize,
    options: &[Opt],
    c: &mut Context,
    next_arg: Option<OsString>,
) -> io::Result<()> {
    next_arg
        // Missing option-argument
        .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))
        .and_then(|val| {
            match (
                c.option_args
                    .iter()
                    .position(|(saved_opt, _)| *saved_opt == index as u16),
                &options[index].kind,
            ) {
                (Some(found), OptArgKind::Multiple) => {
                    // Option was given before

                    if c.arg_ranges[c.option_args[found].1 as usize].end
                        == c.saved_args.len() as u16
                    {
                        c.saved_args.push(val);
                        c.arg_ranges[c.option_args[found].1 as usize]
                            .end += 1;
                        return Ok(());
                    }
                    // Adjust options found after this
                    let mut i = c.arg_ranges
                        [c.option_args[found].1 as usize]
                        .end as usize;

                    c.saved_args.insert(i, val);
                    c.arg_ranges[c.option_args[found].1 as usize].end += 1;

                    while i < c.option_args.len() {
                        match options[c.option_args[i].0 as usize].kind {
                            OptArgKind::Single => {
                                c.option_args[i].1 += 1;
                            }
                            OptArgKind::Multiple => {
                                c.arg_ranges
                                    [c.option_args[i].1 as usize]
                                    .start += 1;
                                c.arg_ranges
                                    [c.option_args[i].1 as usize]
                                    .end += 1;
                            }
                            _ => (),
                        }
                        i += 1;
                    }
                }
                (None, OptArgKind::Multiple) => {
                    // Option not given before
                    c.option_args
                        .push((index as u16, c.arg_ranges.len() as u16));
                    c.arg_ranges.push(
                        c.saved_args.len() as u16
                            ..(c.saved_args.len() + 1) as u16,
                    );
                    c.saved_args.push(val);
                }
                (Some(found), _) => {
                    c.saved_args[c.option_args[found].1 as usize] = val;
                }
                _ => {
                    c.option_args
                        .push((index as u16, c.saved_args.len() as u16));
                    c.saved_args.push(val);
                }
            }
            Ok(())
        })
}

pub fn context_size(c: &Context) {
    use core::mem::{size_of, size_of_val};
    assert_eq!(size_of::<(u16, u16)>(), 4);
    assert_eq!(size_of::<Range<u16>>(), 4);
    assert_eq!(size_of::<TreeNode>(), 4);
    assert_eq!(size_of::<Segment>(), 6);
    assert_eq!(size_of::<(u16, char)>(), 8);

    let counts = &[
        size_of_val(&c),
        size_of_val(&c.operands),
        c.operands.iter().map(|x| size_of_val(x)).sum::<usize>(),
        size_of_val(&c.saved_args),
        c.saved_args.iter().map(|x| size_of_val(x)).sum::<usize>(),
        size_of_val(&c.option_args),
        c.option_args.iter().map(|x| size_of_val(x)).sum::<usize>(),
        size_of_val(&c.arg_ranges),
        c.arg_ranges.iter().map(|x| size_of_val(x)).sum::<usize>(),
        size_of_val(&c.option_occurrences),
        c.option_occurrences.len(),
        size_of_val(&c.selected),
        size_of_val(&c.operands_end),
        size_of_val(&c.path_params),
        // Router sizes
        size_of_val(&c.router),
        size_of_val(&c.router.tree),
        c.router.tree.iter().map(|x| size_of_val(x)).sum::<usize>(),
        size_of_val(&c.router.segments),
        c.router
            .segments
            .iter()
            .map(|x| size_of_val(x))
            .sum::<usize>(),
        size_of_val(&c.router.actions),
        c.router
            .actions
            .iter()
            .map(|x| size_of_val(x))
            .sum::<usize>(),
        size_of_val(&c.router.opt_group_rules),
        c.router.opt_group_rules.len(),
        size_of_val(&c.router.opt_groups),
        c.router
            .opt_groups
            .iter()
            .map(|x| {
                size_of_val(x)
                    + x.iter().map(|x| size_of_val(x)).sum::<usize>()
            })
            .sum::<usize>(),
        size_of_val(&c.router.options),
        c.router
            .options
            .iter()
            .map(|x| size_of_val(x))
            .sum::<usize>(),
        size_of_val(&c.router.short_option_mappers),
        c.router
            .short_option_mappers
            .iter()
            .map(|x| size_of_val(x))
            .sum::<usize>(),
        size_of_val(&c.router.names),
        c.router
            .names
            .iter()
            .map(|x| size_of_val(x) + x.len() * size_of::<char>())
            .sum::<usize>(),
        // size_of_val(&c.router.summaries),
        // c.router
        //     .summaries
        //     .iter()
        //     .map(|x| size_of_val(x) + x.len() * size_of::<char>())
        //     .sum::<usize>(),
        // size_of_val(&c.router.docs),
        // c.router.docs.iter().map(|x| size_of_val(x)).sum::<usize>(),
        size_of_val(&c.router.help_opt_index),
    ];
    println!(
        "Context size: {}
operands size: {}
  sum: {}
saved_args: {}
  sum: {}
option_args: {}
  sum: {}
arg_ranges: {}
  sum: {}
option_occurrences: {}
  sum: {}
selected: {}
operands_end: {}
path_params: {}
--------------------------
Router size: {}
tree: {}
  sum: {}
segments: {}
  sum: {}
actions: {}
  sum: {}
opt_group_rules: {}
  sum: {}
opt_groups: {}
  sum: {}
options: {}
  sum: {}
short_option_mappers: {}
  sum: {}
names: {}
  sum: {}
help_opt_index: {}
--------------------------
Total: {}",
        counts[0],
        counts[1],
        counts[2],
        counts[3],
        counts[4],
        counts[5],
        counts[6],
        counts[7],
        counts[8],
        counts[9],
        counts[10],
        counts[11],
        counts[12],
        counts[13],
        counts[14],
        counts[15],
        counts[16],
        counts[17],
        counts[18],
        counts[19],
        counts[20],
        counts[21],
        counts[22],
        counts[23],
        counts[24],
        counts[25],
        counts[26],
        counts[27],
        counts[28],
        counts[29],
        counts[30],
        counts[31],
        counts.iter().sum::<usize>()
    );
}

/// Find the chunk of code to run, it's options, and
/// it's operands
///
/// Currently, unrecognized options are ignored
pub fn parse_cli_route(
    router: &Router,
    args: impl IntoIterator<Item = OsString>,
) -> io::Result<Context> {
    let mut args = args.into_iter();
    let mut c = Context {
        operands: Vec::new(),
        saved_args: Vec::with_capacity(args.size_hint().0),
        option_args: Vec::<(u16, u16)>::with_capacity(args.size_hint().0),
        arg_ranges: Vec::<Range<u16>>::new(),
        option_occurrences: vec![0; router.options.len()],
        router,
        selected: 0,
        operands_end: 0,
        // The index where path params end and operands
        // begin, indicating how many path params were
        // found
        path_params: 0,
    };
    // Used to validate option groups since key-only options
    // don't add to `option_args`
    let mut options_found = 0u16;
    // Since the first arg, the name of the program,
    // is always skipped we don't need to match on it
    let mut tree_index = 1;

    while let Some(arg) = args.next() {
        let checked_arg = match arg.to_str() {
            Some(a) => a,
            _ => {
                // Won't match any segment or option,
                // since they're all UTF-8, so it can
                // only be an operand
                if router.segments[c.selected as usize].operands
                    != (c.operands.len() - c.path_params as usize) as u16
                {
                    c.operands.push(arg);
                } else
                // Either an option with invalid UTF-8 or an unrecognized
                // segment. Valid options will later obtain option-args
                // without checking UTF-8

                // Will always have bytes because an empty string
                // would've passed UTF-8 checks
                if arg.as_encoded_bytes()[0] == b'-' {
                    // Invalid option
                } else {
                    // Invalid segment
                }
                continue;
            }
        };

        if checked_arg.starts_with('-') {
            let mut chars = checked_arg.chars().skip(1).peekable();

            // Options
            // Unwrap safe because we already checked this index above
            match chars.peek() {
                None => {
                    // Special '-' stdin
                    c.operands.push(arg);
                }
                Some('-') => {
                    if checked_arg.len() == 2 {
                        c.operands_end = c.operands.len() as u16;
                        c.operands.extend(args);
                        break;
                    }
                    // Long

                    #[cfg(feature = "eq-separator")]
                    let (name, eq_index) = match checked_arg.find('=') {
                        // Decision:
                        // We duplicate the range `start` logic since
                        // the resulting `RangeFrom` expression has less
                        // instructions than a `Range` expression
                        Some(eq) => {
                            if eq == 2 || eq == checked_arg.len() - 1 {
                                // ! Error: invalid eq sign
                            }
                            (
                                &checked_arg[{
                                    #[cfg(
                                        feature = "single-hyphen-option-names"
                                    )]
                                    {
                                        1
                                    }
                                    #[cfg(not(
                                        feature = "single-hyphen-option-names"
                                    ))]
                                    {
                                        2
                                    }
                                }
                                    ..eq],
                                Some(eq),
                            )
                        }
                        _ => (
                            &checked_arg[{
                                #[cfg(
                                    feature = "single-hyphen-option-names"
                                )]
                                {
                                    1
                                }
                                #[cfg(not(
                                    feature = "single-hyphen-option-names"
                                ))]
                                {
                                    2
                                }
                            }..],
                            None,
                        ),
                    };
                    #[cfg(not(feature = "eq-separator"))]
                    let name = &checked_arg[{
                        #[cfg(feature = "single-hyphen-option-names")]
                        {
                            1
                        }
                        #[cfg(not(
                            feature = "single-hyphen-option-names"
                        ))]
                        {
                            2
                        }
                    }..];

                    if let Ok(op) =
                        // router.options.iter().position(|mapper| {
                        //     router.names[mapper.name as usize].as_bytes()
                        //         [0]
                        //         == name.as_bytes()[0]
                        //         && router.names[mapper.name as usize]
                        //             == name
                        // })
                        router.options.binary_search_by(|o| {
                                router.names[o.name as usize].cmp(name)
                            })
                    {
                        // Found
                        if c.option_occurrences[op] == 0 {
                            options_found += 1;
                        }
                        c.option_occurrences[op] += 1;
                        if let OptArgKind::KeyOnly =
                            router.options[op].kind
                        {
                        } else {
                            add_found_option(
                                op,
                                router.options,
                                &mut c,
                                #[cfg(feature = "eq-separator")]
                                eq_index
                                    .and_then(|pos| {
                                        Some(checked_arg[pos + 1..].into())
                                    })
                                    .or_else(args.next()),
                                #[cfg(not(feature = "eq-separator"))]
                                args.next(),
                            )?
                        }
                    } else {
                        // Not found
                    }
                }
                _ => {
                    #[cfg(feature = "eq-separator")]
                    let (name, eq_index) = match checked_arg.find('=') {
                        Some(eq) => (&checked_arg[1..eq], Some(eq)),
                        _ => (&checked_arg[1..], None),
                    };
                    #[cfg(all(
                        feature = "single-hyphen-option-names",
                        not(feature = "eq-separator"),
                    ))]
                    let name = &checked_arg[1..];
                    #[cfg(feature = "single-hyphen-option-names")]
                    if let Ok(op) =
                        // router.options.iter().position(|mapper| {
                        //     router.names[mapper.name as usize].as_bytes()
                        //         [0]
                        //         == name.as_bytes()[0]
                        //         && router.names[mapper.name as usize]
                        //             == name
                        // })
                        router.options.binary_search_by(|o| {
                                router.names[o.name as usize].cmp(name)
                            })
                    {
                        // Found
                        if c.option_occurrences[op] == 0 {
                            options_found += 1;
                        }
                        c.option_occurrences[op] += 1;
                        if let OptArgKind::KeyOnly =
                            router.options[op].kind
                        {
                        } else {
                            add_found_option(
                                op,
                                router.options,
                                &mut c,
                                #[cfg(feature = "eq-separator")]
                                eq_index
                                    .and_then(|pos| {
                                        Some(checked_arg[pos + 1..].into())
                                    })
                                    .or_else(args.next()),
                                #[cfg(not(feature = "eq-separator"))]
                                args.next(),
                            )?
                        }
                        continue;
                    }
                    // Shorts
                    for ch in chars {
                        if let Some(o) = router
                            .short_option_mappers
                            .iter()
                            .position(|(_, mapper)| *mapper == ch)
                        {
                            if c.option_occurrences[o] == 0 {
                                options_found += 1;
                            }
                            c.option_occurrences[o] += 1;
                            if let OptArgKind::KeyOnly =
                                router.options[o].kind
                            {
                            } else {
                                // +2 for '-' + character
                                if checked_arg.len() > 2 {
                                    // Found an option that expects an option-arg,
                                    // which maybe shouldn't be allowed in a group
                                    return Err(io::Error::from(
                                        io::ErrorKind::InvalidInput,
                                    ));
                                }
                                add_found_option(
                                    router.short_option_mappers[o].0
                                        as usize,
                                    router.options,
                                    &mut c,
                                    args.next(),
                                )?;
                            }
                        } else {
                            // Option not found
                        }
                    }
                }
            }
            continue;
        }

        if router.segments[c.selected as usize].operands
            != (c.operands.len() - c.path_params as usize) as u16
        {
            c.operands.push(arg);
            continue;
        }

        while tree_index
            < c.selected + router.tree[c.selected as usize].child_span + 1
        {
            if router.names
                [router.segments[tree_index as usize].name as usize]
                .starts_with(':')
            {
                c.selected = tree_index;
                tree_index += 1;
                c.path_params += 1;
                c.operands.push(arg);
                break;
            }
            if checked_arg
                == router.names
                    [router.segments[tree_index as usize].name as usize]
            {
                c.selected = tree_index;
                tree_index += 1;
                break;
            }
            // Skip to next sibling segment
            tree_index += router.tree[tree_index as usize].child_span + 1
        }
    }
    c.saved_args.shrink_to_fit();
    c.option_args.shrink_to_fit();
    c.arg_ranges.shrink_to_fit();

    if c.operands_end == 0 {
        // No terminator was found, so this wasn't set
        c.operands_end = c.operands.len() as u16;
    }

    let groups = router.segments[c.selected as usize].opt_groups >> 12;
    if groups == 0 {
        return Ok(c);
    }
    let index = router.segments[c.selected as usize].opt_groups << 4 >> 4;
    // println!("groups: {}, index: {}", groups, index);
    let groups =
        &router.opt_groups[index as usize..(index + groups) as usize];

    let mut found_opt = None;
    let mut group_options_found = 0u16;
    for (idx, grp) in groups.iter().enumerate() {
        for o in *grp {
            if c.option_occurrences[*o as usize] > 0 {
                found_opt = Some(*o);
                group_options_found += 1;
                if router.opt_group_rules[index as usize + idx] as u8
                    & OptGroupRules::OneOf as u8
                    != 0
                {
                    if let Some(found) = found_opt {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!(
                              "These options are mutually exclusive: {}, {}",
                                router.names[router.options[found as usize].name as usize],
                                router.names[router.options[*o as usize].name as usize]
                            )
                        ));
                    }
                    continue;
                }
                break;
            }
        }
        if found_opt.is_none()
            && router.opt_group_rules[index as usize + idx] as u8
                & OptGroupRules::Required as u8
                != 0
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Missing a required option.",
            ));
        }
    }
    // TODO: This might be wrong as I hit it in benchmark app with
    //       cargo run -- --width 43 --number 2 --number 5
    if group_options_found != options_found {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid options: {}", group_options_found),
        ));
    }
    Ok(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! option_name {
        ($name:literal) => {{
            #[cfg(feature = "single-hyphen-option-names")]
            {
                OsString::from(concat!("-", $name))
            }
            #[cfg(not(feature = "single-hyphen-option-names"))]
            {
                OsString::from(concat!("--", $name))
            }
        }};
    }

    /*
      Cases:
        - runtime error: segment not found
        - runtime error: option not found
        - ? runtime error: option not allowed for this segment (is this necessary?)
        - runtime error: malformed input (URIs)
        - runtime error: option argument validation (bool)
        - runtime error: option argument validation (number)
        - build error: segment name duplicate
        - build error: option name duplicate
        - build error: segment name fails validation
        - build error: option name fails validation
    */
    fn path_action(_: Context) -> io::Result<()> {
        Ok(println!("path command"))
    }
    fn b1_action(_: Context) -> io::Result<()> {
        Ok(println!("path command"))
    }

    fn data() -> Router {
        Router {
            tree: &[
                // 1: path
                TreeNode {
                    child_span: 7,
                    parent: 0,
                },
                // 2:   a
                TreeNode {
                    child_span: 2,
                    parent: 0,
                },
                // 3:     a1
                TreeNode {
                    child_span: 0,
                    parent: 1,
                },
                // 4:     a2
                TreeNode {
                    child_span: 0,
                    parent: 1,
                },
                // 5:   b
                TreeNode {
                    child_span: 2,
                    parent: 0,
                },
                // 6:     b1
                TreeNode {
                    child_span: 0,
                    parent: 4,
                },
                // 7:     b2
                TreeNode {
                    child_span: 0,
                    parent: 4,
                },
                // 8:   c
                TreeNode {
                    child_span: 0,
                    parent: 0,
                },
            ],
            segments: &[
                Segment {
                    operands: 0,
                    opt_groups: 0,
                    name: 3,
                },
                Segment {
                    operands: 0,
                    opt_groups: 0,
                    name: 4,
                },
                Segment {
                    operands: 0,
                    opt_groups: 1 << 12,
                    name: 5,
                },
                Segment {
                    operands: 2,
                    opt_groups: 0,
                    name: 6,
                },
                Segment {
                    operands: 0,
                    opt_groups: 0,
                    name: 7,
                },
                Segment {
                    operands: 0,
                    opt_groups: 0,
                    name: 8,
                },
                Segment {
                    operands: 0,
                    opt_groups: 1 << 12 | 1,
                    name: 9,
                },
                Segment {
                    operands: 0,
                    opt_groups: 0,
                    name: 10,
                },
            ],
            actions: &[
                path_action,
                |_| Ok(println!("a help")),
                |_| Ok(println!("a1 help")),
                |_| Ok(println!("a2 help")),
                |_| Ok(println!("b help")),
                b1_action,
                |_| Ok(println!("b2 help")),
                |_| Ok(println!("c help")),
            ],
            short_option_mappers: &[(0, 'k'), (1, 'm'), (2, 's')],
            names: &[
                "key-only", "multi1", "single1", "path", "a", "a1", "a2",
                "b", "b1", "b2", "c",
            ],
            options: &[
                Opt {
                    kind: OptArgKind::KeyOnly,
                    name: 0,
                },
                Opt {
                    kind: OptArgKind::Multiple,
                    name: 1,
                },
                Opt {
                    kind: OptArgKind::Single,
                    name: 2,
                },
            ],
            opt_group_rules: &[
                OptGroupRules::AnyOf as u8,
                OptGroupRules::Required as u8,
            ],
            opt_groups: &[&[1, 2], &[0]],
            help_opt_index: None,
        }
    }

    #[test]
    fn should_parse_a_cli_route_with_no_options_or_terminator() {
        let router = data();

        let c =
            parse_cli_route(&router, vec![OsString::from("c")]).unwrap();
        assert_eq!(c.selected, 7);
    }
    #[test]
    fn should_parse_a_cli_route_with_options() {
        let router = data();

        // * An option that expects no option-args
        let c = parse_cli_route(
            &router,
            vec![
                OsString::from("b"),
                option_name!("key-only"),
                OsString::from("b1"),
            ],
        )
        .unwrap();
        assert_eq!(c.selected, 5);
        assert_eq!(c.option_occurrences, [1, 0, 0]);

        // * Ignore double-hyphon option names
        #[cfg(feature = "single-hyphen-option-names")]
        {
            let c = parse_cli_route(
                &router,
                vec![
                    OsString::from("b"),
                    OsString::from("--key-only"),
                    OsString::from("b1"),
                ],
            )
            .unwrap();
            assert_eq!(c.selected, 5);
            assert_eq!(c.option_occurrences, [0, 0, 0]);
        }

        // * An option that expects an option-arg separated
        // * by an '=' character
        #[cfg(feature = "eq-separator")]
        {
            let c = parse_cli_route(
                &router,
                vec![
                    OsString::from("b"),
                    option_name!("single1=val"),
                    OsString::from("b1"),
                ],
            )
            .unwrap();
            assert_eq!(c.selected, 5);
            assert_eq!(c.option_occurrences, [0, 1, 0]);
            assert_eq!(c.saved_args, vec![OsString::from("val")]);

            // TODO: Handle "-=" and "-=val" case
        }

        // * Treat the '=' separator in option names as a
        // * regular character
        #[cfg(not(feature = "eq-separator"))]
        {
            let c = parse_cli_route(
                &router,
                vec![
                    OsString::from("b"),
                    option_name!("cal=val"),
                    OsString::from("b1"),
                ],
            )
            .unwrap();
            assert_eq!(c.selected, 5);
            assert_eq!(c.option_occurrences, [0, 0, 0]);
            assert_eq!(c.saved_args.len(), 0);
        }

        // * Prohibit an option from clustering when it expects
        // * an option-arg
        let c = parse_cli_route(
            &router,
            vec![
                OsString::from("b"),
                OsString::from("-skm"),
                OsString::from("b1"),
            ],
        );
        assert!(c.is_err());

        // * An option that expects an option-arg
        let c = parse_cli_route(
            &router,
            vec![
                OsString::from("b"),
                option_name!("single1"),
                OsString::from("val"),
                OsString::from("b1"),
            ],
        )
        .unwrap();
        assert_eq!(c.selected, 5);
        assert_eq!(c.option_occurrences, [0, 0, 1]);
        assert_eq!(c.saved_args, vec![OsString::from("val")]);

        // * An option that expects an option-arg and can
        // * occur multiple times
        let c = parse_cli_route(
            &router,
            vec![
                OsString::from("b"),
                option_name!("multi1"),
                OsString::from("val"),
                OsString::from("b1"),
                option_name!("single1"),
                OsString::from("single1-val"),
                option_name!("multi1"),
                OsString::from("val2"),
            ],
        )
        .unwrap();
        assert_eq!(c.selected, 5);
        assert_eq!(c.option_occurrences, [0, 2, 1]);
        assert_eq!(
            c.saved_args,
            vec![
                OsString::from("val"),
                OsString::from("val2"),
                OsString::from("single1-val")
            ],
        );
        assert_eq!(c.arg_ranges.len(), 1);
        assert_eq!(c.arg_ranges[0].start, 0);
        assert_eq!(c.arg_ranges[0].end, 2);
        assert_eq!(c.option_args, vec![(1, 0), (2, 2)]);

        // * An option that expects an option-arg and can
        // * replaces its first occurrence
        let c = parse_cli_route(
            &router,
            vec![
                OsString::from("b"),
                option_name!("single1"),
                OsString::from("single1-val"),
                // option_name!("multi1"),
                // OsString::from("val"),
                OsString::from("b1"),
                option_name!("single1"),
                OsString::from("single1-val2"),
                // option_name!("multi1"),
                // OsString::from("val2"),
            ],
        )
        .unwrap();
        assert_eq!(c.selected, 5);
        assert_eq!(c.option_occurrences, [0, 0, 2]);
        assert_eq!(
            c.saved_args,
            vec![
                // OsString::from("val"),
                // OsString::from("val2"),
                OsString::from("single1-val2")
            ],
        );
        assert_eq!(c.arg_ranges.len(), 0);
        assert_eq!(c.option_args, vec![(2, 0)]);

        // * Short option aliases
        let c = parse_cli_route(
            &router,
            vec![
                OsString::from("b"),
                OsString::from("-s"),
                OsString::from("val"),
                OsString::from("b1"),
                OsString::from("-k"),
                OsString::from("-k"),
            ],
        )
        .unwrap();
        assert_eq!(c.selected, 5);
        assert_eq!(c.option_occurrences, [2, 0, 1]);
        assert_eq!(c.saved_args, vec![OsString::from("val")]);
    }
    #[test]
    fn should_parse_a_cli_route_with_terminator() {
        let router = data();

        let c = parse_cli_route(
            &router,
            vec![
                OsString::from("b"),
                OsString::from("--"),
                OsString::from("--single1"),
                OsString::from("b1"),
            ],
        )
        .unwrap();
        assert_eq!(c.selected, 4);
        assert_eq!(c.option_occurrences, [0, 0, 0]);
        assert_eq!(c.operands_end, 2);
        assert_eq!(
            c.operands,
            vec![OsString::from("--single1"), OsString::from("b1")]
        );
    }
    #[test]
    fn should_parse_a_cli_route_with_operands() {
        let router = data();
        let c = parse_cli_route(
            &router,
            vec![
                OsString::from("a"),
                option_name!("key-only"),
                OsString::from("a2"),
                OsString::from("operand1"),
                OsString::from("operand2"),
                option_name!("multi1"),
                OsString::from("some-multi-opt-val"),
                option_name!("multi1"),
                OsString::from("another-multi-opt-val"),
                option_name!("single1"),
                OsString::from("single-opt-val"),
            ],
        )
        .unwrap();
        assert_eq!(c.selected, 3);
        assert_eq!(
            c.saved_args,
            vec![
                OsString::from("some-multi-opt-val"),
                OsString::from("another-multi-opt-val"),
                OsString::from("single-opt-val"),
            ]
        );
        assert_eq!(
            c.operands,
            vec![OsString::from("operand1"), OsString::from("operand2")]
        );
    }
    #[test]
    fn should_validate_option_against_option_groups() {
        let router = data();
        // let group_count = 5;
        // let index = 677u16;
        // let composed = group_count << 12 | index;

        // println!(
        //     "n: {}, groups: {}, index: {}",
        //     composed,
        //     composed >> 12,
        //     composed << 4 >> 4
        // );
        if parse_cli_route(
            &router,
            vec![
                OsString::from("a"),
                OsString::from("a1"),
                option_name!("key-only"),
            ],
        )
        .is_ok()
        {
            panic!("Can have `single1` or `multi1` options, but not `key-only` option");
        }

        assert!(parse_cli_route(
            &router,
            vec![
                OsString::from("a"),
                OsString::from("a1"),
                option_name!("single1"),
                OsString::from("single-val"),
            ],
        )
        .is_ok());
        assert!(parse_cli_route(
            &router,
            vec![OsString::from("a"), OsString::from("a1")],
        )
        .is_ok());
        assert!(
            parse_cli_route(&router, vec![OsString::from("a")]).is_ok()
        );
        assert!(parse_cli_route(
            &router,
            vec![OsString::from("b"), OsString::from("b2")]
        )
        .is_err());
        assert!(parse_cli_route(
            &router,
            vec![
                OsString::from("b"),
                OsString::from("b2"),
                option_name!("key-only")
            ]
        )
        .is_ok());
    }
}

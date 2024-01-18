/// # Features
/// * **eq-separator** -
///   Allow separating options from option-arguments
///   with a '='
/// * **single-hyphen-option-names** -
///   Changes options to expect a single "-" prefix
///   instead of "--", and short options are disabled
mod uri;
use {
    std::{io, ops::Range},
    tree_pack::TreePack,
    uri::*,
};

pub type Action = fn(c: Context) -> io::Result<()>;

pub enum RouteKind {
    CLI,
    URL,
}

pub struct Router {
    pub tree: TreePack,
    pub segments: &'static [Seg],
    pub actions: &'static [Action],
    // How many times the option was found
    // option_occurrences: &mut [u8],
    pub options: &'static [Opt],
    pub short_option_mappers: &'static [(u16, char)],
    pub names: &'static [&'static str],
}
/// Things needed at runtime for the segment (summary stored separately)
pub struct Seg {
    // action: u8,
    /// First bit(s) tell whether this segment has multiple
    /// groups. The next bits are for an offset, from the index
    /// indicating how many groups there are. The remaining bits
    /// are for the index.
    pub opt_groups: u16,
    /// An index into the shared list of names
    pub name: u16,
}
/// Used during parsing to determine if it needs to be cached
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
/// Holds data necessary to map a parsed argument to an option
pub struct Opt {
    pub kind: OptArgKind,
    /// An index into the shared list of names
    pub name: u16,
}
pub struct Context {
    selected: u16,
    pub option_occurrences: Vec<u8>,
    // Raw args, and operands will be attached to the
    // end
    pub saved_args: Vec<String>,
    // Index to options and index to saved_args
    pub option_args: Vec<(u16, u16)>,
    // For options that expect multiple args.
    // When there are multiple args for an option, they
    // should appear next to each other in `saved_args`
    pub arg_ranges: Vec<Range<u16>>,
}

fn add_found_option(
    index: usize,
    options: &[Opt],
    c: &mut Context,
    next_arg: Option<String>,
) -> io::Result<()> {
    c.option_occurrences[index] += 1;
    next_arg
        // Missing option-argument
        .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))
        .and_then(|val| {
            if let OptArgKind::Multiple = options[index].kind {
                // Find previous occurrence for this option
                match c
                    .option_args
                    .iter()
                    .position(|(saved_opt, _)| *saved_opt == index as u16)
                {
                    Some(found) => {
                        // Option was given before
                        if c.arg_ranges[c.option_args[found].1 as usize].end
                            >= c.saved_args.len() as u16
                        {
                            c.saved_args.push(val);
                        } else {
                            c.saved_args.insert(
                                c.arg_ranges[c.option_args[found].1 as usize].end as usize,
                                val,
                            );
                            c.arg_ranges[c.option_args[found].1 as usize].end += 1;
                            // Adjust options found after this
                            let mut i = c.arg_ranges[c.option_args[found].1 as usize].end as usize;
                            while i < c.option_args.len() {
                                match options[c.option_args[i].0 as usize].kind {
                                    OptArgKind::Single => {
                                        c.option_args[i].1 += 1;
                                    }
                                    OptArgKind::Multiple => {
                                        c.arg_ranges[c.option_args[i].0 as usize].start += 1;
                                        c.arg_ranges[c.option_args[i].0 as usize].end += 1;
                                    }
                                    _ => (),
                                }
                                i += 1;
                            }
                        }
                    }
                    _ => {
                        // Option not given before
                        c.option_args
                            .push((index as u16, c.arg_ranges.len() as u16));
                        c.arg_ranges
                            .push(c.saved_args.len() as u16..(c.saved_args.len() + 1) as u16);
                        c.saved_args.push(val);
                    }
                }
            } else {
                c.option_args
                    .push((index as u16, c.saved_args.len() as u16));
                c.saved_args.push(val);
            }
            Ok(())
        })
}

// #[inline]
// fn set_option_if_found(
//     c: &mut Context,
//     arg: &str,
//     next_arg: Option<String>,
//     options: &[Opt],
//     names: &[&str],
// ) -> Option<io::Result<()>> {
//     options
//         .iter()
//         .position(|mapper| names[mapper.name as usize] == arg)
//         .and_then(|o| Some(add_found_option(o, options, c, next_arg)))
// }

/// Find the chunk of code to run, it's options, and
/// it's operands
///
/// Currently, unrecognized options are ignored
pub fn parse_route(
    // tree: &TreePack,
    // segments: &[Seg],
    // // How many times the option was found
    // // option_occurrences: &mut [u8],
    // options: &[Opt],
    // short_option_mappers: &[(u16, char)],
    // names: &[&str],
    router: &Router,
    kind: RouteKind,
    args: impl IntoIterator<Item = String>,
) -> io::Result<Context> {
    let mut args = args.into_iter();
    let mut c = Context {
        selected: 0,
        option_occurrences: vec![0; router.options.len()],
        saved_args: Vec::with_capacity(args.size_hint().0),
        option_args: Vec::<(u16, u16)>::with_capacity(args.size_hint().0),
        arg_ranges: Vec::<Range<u16>>::new(),
    };
    // Since the first arg, the name of the
    // program, is always skipped we don't
    // need to match on it
    let mut tree_index = 1;
    let mut terminated = false;

    match kind {
        RouteKind::CLI => {
            // TODO: Options can be in the form of "-key=val"
            // let mut equal_sign = 0;
            while let Some(arg) = args.next() {
                if terminated {
                    c.saved_args.push(arg);
                    continue;
                }
                if arg.starts_with('-') {
                    // Consider moving this down to the below features
                    // since it's a more rare case
                    if arg.len() == 1 {
                        // Special
                    }
                    let mut chars = arg.chars().skip(1).peekable();

                    // Options
                    // Unwrap safe because we already checked this index above
                    if *chars.peek().unwrap() == '-' {
                        if arg.len() == 2 {
                            terminated = true;
                            continue;
                        }
                        // Long

                        #[cfg(feature = "eq-separator")]
                        let key_val = match arg.find('=') {
                            // Decision:
                            // We duplicate the range `start` logic since
                            // the resulting `RangeFrom` expression has less
                            // instructions than a `Range` expression
                            Some(eq) => (
                                &arg[{
                                    #[cfg(feature = "single-hyphen-option-names")]
                                    {
                                        1
                                    }
                                    #[cfg(not(feature = "single-hyphen-option-names"))]
                                    {
                                        2
                                    }
                                }..eq],
                                Some(eq),
                            ),
                            _ => (
                                &arg[{
                                    #[cfg(feature = "single-hyphen-option-names")]
                                    {
                                        1
                                    }
                                    #[cfg(not(feature = "single-hyphen-option-names"))]
                                    {
                                        2
                                    }
                                }..],
                                None,
                            ),
                        };
                        if let Some(op) = router.options.iter().position(|mapper| {
                            router.names[mapper.name as usize] == {
                                #[cfg(feature = "eq-separator")]
                                {
                                    key_val.0
                                }
                                #[cfg(not(feature = "eq-separator"))]
                                {
                                    &arg[{
                                        #[cfg(feature = "single-hyphen-option-names")]
                                        {
                                            1
                                        }
                                        #[cfg(not(feature = "single-hyphen-option-names"))]
                                        {
                                            2
                                        }
                                    }..]
                                }
                            }
                        }) {
                            // Found
                            if let OptArgKind::KeyOnly = router.options[op].kind {
                                // Or, store in `option_args` as (o, 0)
                                c.option_occurrences[op] += 1;
                            } else {
                                add_found_option(
                                    op,
                                    router.options,
                                    &mut c,
                                    #[cfg(feature = "eq-separator")]
                                    key_val
                                        .1
                                        .and_then(|pos| Some(arg[pos + 1..].to_owned()))
                                        .or_else(|| args.next()),
                                    #[cfg(not(feature = "eq-separator"))]
                                    args.next(),
                                )?
                            }
                        } else {
                            // Not found
                        }
                    } else {
                        #[cfg(feature = "eq-separator")]
                        let key_val = match arg.find('=') {
                            Some(eq) => (&arg[1..eq], Some(eq)),
                            _ => (&arg[1..], None),
                        };
                        #[cfg(feature = "single-hyphen-option-names")]
                        if let Some(op) = router.options.iter().position(|mapper| {
                            router.names[mapper.name as usize] == {
                                #[cfg(feature = "eq-separator")]
                                {
                                    key_val.0
                                }
                                #[cfg(not(feature = "eq-separator"))]
                                {
                                    &arg[1..]
                                }
                            }
                        }) {
                            // Found
                            if let OptArgKind::KeyOnly = router.options[op].kind {
                                // Or, store in `option_args` as (o, 0)
                                c.option_occurrences[op] += 1;
                            } else {
                                add_found_option(
                                    op,
                                    router.options,
                                    &mut c,
                                    #[cfg(feature = "eq-separator")]
                                    key_val
                                        .1
                                        .and_then(|pos| Some(arg[pos + 1..].to_owned()))
                                        .or_else(|| args.next()),
                                    #[cfg(not(feature = "eq-separator"))]
                                    args.next(),
                                )?
                            }
                            continue;
                        }
                        // #[cfg(feature = "single-hyphen-option-names")]
                        // if let Some(res) = set_option_if_found(
                        //     &mut c,
                        //     &arg[1..],
                        //     &mut args,
                        //     router.options,
                        //     router.names,
                        // ) {
                        //     res?;
                        //     continue;
                        // }

                        // Shorts
                        for ch in chars {
                            if let Some(o) = router
                                .short_option_mappers
                                .iter()
                                .position(|mapper| mapper.1 == ch)
                            {
                                if let OptArgKind::KeyOnly = router.options[o].kind {
                                    // Or, store in `option_args` as (o, 0)
                                    c.option_occurrences[o] += 1;
                                } else
                                // +2 for '-' + character
                                if arg.len() > 2 {
                                    // Found an option that expects an option-arg,
                                    // which maybe shouldn't be allowed
                                    return Err(io::Error::from(io::ErrorKind::InvalidInput));
                                } else {
                                    add_found_option(
                                        router.short_option_mappers[o].0 as usize,
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
                    continue;
                }
                while tree_index
                    < c.selected
                        + router.tree.descendents_len(c.selected as usize).unwrap() as u16
                        + 1
                {
                    if router.names[router.segments[tree_index as usize].name as usize]
                        .starts_with(':')
                        || arg == router.names[router.segments[tree_index as usize].name as usize]
                    {
                        c.selected = tree_index;
                        tree_index += 1;
                        break;
                    }
                    // Skip to next sibling segment
                    tree_index +=
                        router.tree.descendents_len(tree_index as usize).unwrap() as u16 + 1
                }
            }
        }
        RouteKind::URL => {
            if let Some(route) = args.next() {
                let route_bytes = route.as_bytes();
                let lengths = parse_uri_scheme_and_authority(route_bytes);
                let mut index = lengths.scheme_len as usize + "://".len();
                if lengths.user_len > 0 {
                    // +1 for the '@'
                    index += lengths.user_len as usize + 1;
                }
                // Assumes there will always be a Host component
                index += lengths.host_len as usize;
                if lengths.port_len > 0 {
                    // +1 for the ':', and another because we've either
                    // landed on '/' before, or there was no path
                    index += lengths.port_len as usize + 1;
                }
                if index == route_bytes.len() {
                    return Ok(c);
                }
                if route_bytes[index] == b'/' {
                    index += 1;
                }

                let mut start = index;
                // let mut dividers = route[index..].match_indices(['/', '?']);
                while start < route_bytes.len() {
                    if index == route_bytes.len()
                        || route_bytes[index] == b'/'
                        || route_bytes[index] == b'?'
                        || route_bytes[index] == b'#'
                    {
                        if start == index {
                            break;
                        }
                        let arg = if index == route_bytes.len() {
                            &route[start..]
                        } else {
                            &route[start..index]
                        };

                        while tree_index
                            < c.selected
                                + router.tree.descendents_len(c.selected as usize).unwrap() as u16
                                + 1
                        {
                            if router.names[router.segments[tree_index as usize].name as usize]
                                .starts_with(':')
                                || arg
                                    == router.names
                                        [router.segments[tree_index as usize].name as usize]
                            {
                                c.selected = tree_index;
                                tree_index += 1;
                                start = index + 1;
                                break;
                            }
                            // Skip to next sibling segment
                            tree_index +=
                                router.tree.descendents_len(tree_index as usize).unwrap() as u16 + 1
                        }
                        if index == route_bytes.len() {
                            return Ok(c);
                        }
                        if route_bytes[index] != b'/' {
                            break;
                        }
                    }
                    index += 1;
                }
                index += 1;
                start = index;
                let mut equal_sign = 0;
                while index < route_bytes.len() {
                    match route_bytes[index] {
                        b'&' => {
                            if equal_sign > 0 {
                                let opt = &route[start..start + equal_sign];
                                // Key-value pair
                                if let Some(o) = router
                                    .options
                                    .iter()
                                    .position(|mapper| router.names[mapper.name as usize] == opt)
                                {
                                    c.option_occurrences[o] += 1;
                                } else {
                                    // Not found
                                }
                                equal_sign = 0;
                            } else {
                                if let Some(o) = router.options.iter().position(|mapper| {
                                    router.names[mapper.name as usize] == &route[start..]
                                }) {
                                    c.option_occurrences[o] += 1;
                                } else {
                                    // Not found
                                }
                            }
                            start = index + 1;
                        }
                        b'#' => {
                            break;
                        }
                        b'=' => {
                            equal_sign = index - start;
                        }
                        _ => (),
                    }
                    index += 1;
                }
                if start < route_bytes.len() {
                    if equal_sign > 0 {
                        let opt = &route[start..start + equal_sign];
                        // Key-value pair
                        if let Some(o) = router
                            .options
                            .iter()
                            .position(|mapper| router.names[mapper.name as usize] == opt)
                        {
                            c.option_occurrences[o] += 1;
                        } else {
                            // Not found
                        }
                    } else {
                        if let Some(o) = router.options.iter().position(|mapper| {
                            router.names[mapper.name as usize] == &route[start..index]
                        }) {
                            c.option_occurrences[o] += 1;
                        } else {
                            // Not found
                        }
                    }
                }
            }
        }
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
                String::from(concat!("-", $name))
            }
            #[cfg(not(feature = "single-hyphen-option-names"))]
            {
                String::from(concat!("--", $name))
            }
        }};
    }

    /// Make an option name that's either prefixed with one
    /// or two hyphens, depending on if the feature's enabled
    // const fn option_name(name: &str) -> String {
    //     #[cfg(feature = "single-hyphen-option-names")]
    //     return String::from_iter(["-", name]);
    //     #[cfg(not(feature = "single-hyphen-option-names"))]
    //     return String::from_str("--").unwrap().pu;
    // }
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

    fn data() -> (Router, Vec<&'static str>) {
        // enum O {
        //     KeyOnly = 0,
        //     Single1,
        //     Multi1,
        // }
        let mut router = Router {
            tree: TreePack::new(),
            segments: &[
                Seg {
                    opt_groups: 0,
                    name: 0,
                },
                Seg {
                    opt_groups: 0,
                    name: 1,
                },
                Seg {
                    opt_groups: 0,
                    name: 2,
                },
                Seg {
                    opt_groups: 0,
                    name: 3,
                },
                Seg {
                    opt_groups: 0,
                    name: 4,
                },
                Seg {
                    opt_groups: 0,
                    name: 5,
                },
                Seg {
                    opt_groups: 0,
                    name: 6,
                },
                Seg {
                    opt_groups: 0,
                    name: 7,
                },
                Seg {
                    opt_groups: 0,
                    name: 8,
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
            short_option_mappers: &[(0, 'k'), (1, 's'), (2, 'm')],
            names: &[
                "root", "path", "a", "a1", "a2", "b", "b1", "b2", "c", "key-only", "single1",
                "multi1",
            ],
            options: &[
                Opt {
                    kind: OptArgKind::KeyOnly,
                    name: 9,
                },
                Opt {
                    kind: OptArgKind::Single,
                    name: 10,
                },
                Opt {
                    kind: OptArgKind::Multiple,
                    name: 11,
                },
            ],
        };
        let summaries = vec![
            "root summary",
            "path summary",
            "a summary",
            "a1 summary",
            "a2 summary",
            "b summary",
            "b1 summary",
            "b2 summary",
            "c summary",
            "key-only summary",
            "single1 summary",
            "multi1 summary",
        ];

        router.tree.insert(0); // 1: path
        router.tree.insert(1); // 2:   a
        router.tree.insert(2); // 3:     a1
        router.tree.insert(2); // 4:     a2
        router.tree.insert(1); // 5:   b
        router.tree.insert(5); // 6:     b1
        router.tree.insert(5); // 7:     b2
        router.tree.insert(1); // 8:   c

        (router, summaries)
    }

    #[test]
    fn should_parse_a_url_route_with_no_options_or_fragment() {
        let (router, _) = data();

        // URL route with yes path, no options, no fragment
        let c = parse_route(
            &router,
            RouteKind::URL,
            vec!["https://example.com:443/path/a/a2".to_string()],
        )
        .unwrap();
        assert_eq!(c.selected, 4);

        // URL route with no path, no options, no fragment
        let c = parse_route(
            &router,
            RouteKind::URL,
            vec!["https://example.com:443".to_string()],
        )
        .unwrap();
        assert_eq!(c.selected, 0);

        // Doesn't exist, `index` is the last recognized segment
        let c = parse_route(
            &router,
            RouteKind::URL,
            vec!["https://example.com:443/path/b/b3".to_string()],
        )
        .unwrap();
        assert_eq!(c.selected, 5);
    }
    #[test]
    fn should_parse_a_url_route_with_options() {
        let (router, _) = data();

        // URL route with yes path, yes options, no fragment
        let c = parse_route(
            &router,
            RouteKind::URL,
            vec!["https://example.com:443/path/a/a2?single1=abc&key-only".to_string()],
        )
        .unwrap();
        assert_eq!(c.selected, 4);
        assert_eq!(c.option_occurrences, [1, 1, 0]);

        // URL route with no path, yes options, no fragment
        let c = parse_route(
            &router,
            RouteKind::URL,
            vec!["https://example.com:443?single1=0".to_string()],
        )
        .unwrap();
        assert_eq!(c.selected, 0);
        assert_eq!(c.option_occurrences, [0, 1, 0]);
    }
    // TODO: Figure out how a fragment fits into this segments and options concept
    #[test]
    fn should_parse_a_url_route_with_a_fragment() {
        let (router, _) = data();

        // URL route with no path, no options, yes fragment
        let c = parse_route(
            &router,
            RouteKind::URL,
            vec!["https://example.com:443?#frag".to_string()],
        )
        .unwrap();
        assert_eq!(c.selected, 0);
        assert_eq!(c.option_occurrences, [0, 0, 0]);

        // URL route with no path, yes options, yes fragment
        let c = parse_route(
            &router,
            RouteKind::URL,
            vec!["https://example.com:443?key-only#frag".to_string()],
        )
        .unwrap();
        assert_eq!(c.selected, 0);
        assert_eq!(c.option_occurrences, [1, 0, 0]);

        // URL route with yes path, no options, yes fragment
        let c = parse_route(
            &router,
            RouteKind::URL,
            vec!["https://example.com:443/path/a/a2#frag".to_string()],
        )
        .unwrap();
        assert_eq!(c.selected, 4);
        assert_eq!(c.option_occurrences, [0, 0, 0]);

        // URL route with yes path, yes options, yes fragment
        let c = parse_route(
            &router,
            RouteKind::URL,
            vec!["https://example.com:443/path/a/a2?key-only#frag".to_string()],
        )
        .unwrap();
        assert_eq!(c.selected, 4);
        assert_eq!(c.option_occurrences, [1, 0, 0]);
    }
    #[test]
    fn should_parse_a_cli_route_with_no_options_or_terminator() {
        let (router, _) = data();

        let c = parse_route(
            &router,
            RouteKind::CLI,
            vec!["path".to_string(), "c".to_string()],
        )
        .unwrap();
        assert_eq!(c.selected, 8);
    }
    #[test]
    fn should_parse_a_cli_route_with_options() {
        let (router, _) = data();

        // * An option that expects no option-args
        let c = parse_route(
            &router,
            RouteKind::CLI,
            vec![
                "path".to_string(),
                "b".to_string(),
                option_name!("key-only"),
                "b1".to_string(),
            ],
        )
        .unwrap();
        assert_eq!(c.selected, 6);
        assert_eq!(c.option_occurrences, [1, 0, 0]);

        // * Ignore double-hyphon option names
        #[cfg(feature = "single-hyphen-option-names")]
        {
            let c = parse_route(
                &router,
                RouteKind::CLI,
                vec![
                    "path".to_string(),
                    "b".to_string(),
                    "--key-only".to_string(),
                    "b1".to_string(),
                ],
            )
            .unwrap();
            assert_eq!(c.selected, 6);
            assert_eq!(c.option_occurrences, [0, 0, 0]);
        }

        // * An option that expects an option-arg separated
        // * by an '=' character
        #[cfg(feature = "eq-separator")]
        {
            let c = parse_route(
                &router,
                RouteKind::CLI,
                vec![
                    "path".to_string(),
                    "b".to_string(),
                    option_name!("single1=val"),
                    "b1".to_string(),
                ],
            )
            .unwrap();
            assert_eq!(c.selected, 6);
            assert_eq!(c.option_occurrences, [0, 1, 0]);
            assert_eq!(c.saved_args, vec!["val".to_string()]);

            // TODO: Handle "-=" and "-=val" case
        }

        // * Treat the '=' separator in option names as a
        // * regular character
        #[cfg(not(feature = "eq-separator"))]
        {
            let c = parse_route(
                &router,
                RouteKind::CLI,
                vec![
                    "path".to_string(),
                    "b".to_string(),
                    option_name!("cal=val"),
                    "b1".to_string(),
                ],
            )
            .unwrap();
            assert_eq!(c.selected, 6);
            assert_eq!(c.option_occurrences, [0, 0, 0]);
            assert_eq!(c.saved_args.len(), 0);
        }

        // * Prohibit an option from clustering when it expects
        // * an option-arg
        let c = parse_route(
            &router,
            RouteKind::CLI,
            vec![
                "path".to_string(),
                "b".to_string(),
                "-skm".to_string(),
                "b1".to_string(),
            ],
        );
        assert!(c.is_err());

        // * An option that expects an option-arg
        let c = parse_route(
            &router,
            RouteKind::CLI,
            vec![
                "path".to_string(),
                "b".to_string(),
                option_name!("single1"),
                "val".to_string(),
                "b1".to_string(),
            ],
        )
        .unwrap();
        assert_eq!(c.selected, 6);
        assert_eq!(c.option_occurrences, [0, 1, 0]);
        assert_eq!(c.saved_args, vec!["val".to_string()]);

        // * An option that expects an option-arg and can
        // * occur multiple times
        let c = parse_route(
            &router,
            RouteKind::CLI,
            vec![
                "path".to_string(),
                "b".to_string(),
                option_name!("multi1"),
                "val".to_string(),
                "b1".to_string(),
                option_name!("multi1"),
                "val2".to_string(),
            ],
        )
        .unwrap();
        assert_eq!(c.selected, 6);
        assert_eq!(c.option_occurrences, [0, 0, 2]);
        assert_eq!(c.saved_args, vec!["val".to_string(), "val2".to_string()]);
        assert_eq!(c.arg_ranges.len(), 1);
        assert_eq!(c.arg_ranges[0].start, 0);
        assert_eq!(c.arg_ranges[0].end, 1);

        // * Short option aliases
        let c = parse_route(
            &router,
            RouteKind::CLI,
            vec![
                "path".to_string(),
                "b".to_string(),
                "-s".to_string(),
                "val".to_string(),
                "b1".to_string(),
                "-k".to_string(),
                "-k".to_string(),
            ],
        )
        .unwrap();
        assert_eq!(c.selected, 6);
        assert_eq!(c.option_occurrences, [2, 1, 0]);
        assert_eq!(c.saved_args, vec!["val".to_string()]);
    }
    #[test]
    fn should_terminate_a_cli_route_and_store_operands() {
        let (router, _) = data();

        let c = parse_route(
            &router,
            RouteKind::CLI,
            vec![
                "path".to_string(),
                "b".to_string(),
                "--".to_string(),
                "--single1".to_string(),
                "b1".to_string(),
            ],
        )
        .unwrap();
        assert_eq!(c.selected, 5);
        assert_eq!(c.option_occurrences, [0, 0, 0]);
        assert_eq!(
            c.saved_args,
            vec!["--single1".to_string(), "b1".to_string()]
        );
    }
    // #[test]
    // fn should_accept_procedure_operands_in_a_cli_route(){

    // }
    #[test]
    fn should_parse_placeholder_segments() {
        // enum O {
        //     KeyOnly,
        //     Single1,
        //     Multi1,
        // }
        let mut router = Router {
            tree: TreePack::new(),
            segments: &[
                Seg {
                    opt_groups: 0,
                    name: 0,
                },
                Seg {
                    opt_groups: 0,
                    name: 1,
                },
                Seg {
                    opt_groups: 0,
                    name: 2,
                },
                Seg {
                    opt_groups: 0,
                    name: 3,
                },
                Seg {
                    opt_groups: 0,
                    name: 4,
                },
                Seg {
                    opt_groups: 0,
                    name: 3,
                },
            ],
            actions: &[
                |_| Ok(println!("program help")),
                |_| Ok(println!("built-in help")),
                |_| Ok(println!("fun help")),
                |_| Ok(println!("extension help")),
                |_| Ok(println!("fun help")),
            ],
            options: &[
                Opt {
                    kind: OptArgKind::KeyOnly,
                    name: 5,
                },
                Opt {
                    kind: OptArgKind::Single,
                    name: 6,
                },
                Opt {
                    kind: OptArgKind::Multiple,
                    name: 7,
                },
            ],
            short_option_mappers: &[(0, 'a'), (1, 'b'), (2, 'c')],
            names: &[
                "root",
                "program",
                "built-in",
                "fun",
                ":extension",
                /* "other-built-in", */
                "key-only",
                "single1",
                "multi1",
            ],
        };
        // let summaries = vec![
        //     "root summary",
        //     "program summary",
        //     "built-in summary",
        //     "fun summary",
        //     "extension summary",
        //     "extension fun summary",
        //     // "other-built-in summary",
        //     "key-only summary",
        //     "single1 summary",
        //     "multi1 summary",
        // ];

        router.tree.insert(0); // 1: program
        router.tree.insert(1); // 2:   built-in
        router.tree.insert(2); // 3:     fun
        router.tree.insert(1); // 4:   :extension
        router.tree.insert(4); // 5:     fun

        // c.tree.insert(1); // 6:   other-built-in

        let c = parse_route(
            &router,
            RouteKind::URL,
            vec!["https://example.com:443/program/built-in/fun".to_string()],
        )
        .unwrap();
        assert_eq!(c.selected, 3);

        let c = parse_route(
            &router,
            RouteKind::URL,
            vec!["https://example.com:443/program/extensionA/fun".to_string()],
        )
        .unwrap();
        assert_eq!(c.selected, 5);
    }
}

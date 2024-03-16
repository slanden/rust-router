use {
    common::*,
    std::{io, ops::Range},
};

enum URIState {
    Scheme,
    User,
    IPv6,
    Host,
    Port,
    // Path,
    // Query,
    // Fragment,
}

/// Each field represents the length of its slice in
/// the source string.
///
/// The start of a certain component in the URI is
/// found by adding the previous components' lengths
/// and ignored characters up to the component of
/// interest.
///
/// ## Example
/// "https://example.com" will have `scheme_end = 5`
/// and a `host_end = 11`. "example.com" starts at
/// `scheme_end + "://".len()`
pub struct URIAuthorityLengths {
    pub scheme_len: u8,
    pub user_len: u8,
    pub host_len: u8,
    pub port_len: u8,
}

pub fn parse_uri_route<'a>(
    router: &'a Router,
    route: &'a str,
) -> io::Result<Context<'a>> {
    let mut c = Context {
        router,
        selected: 0,
        operands: Vec::new(),
        operands_end: 0,
        option_occurrences: vec![0; router.options.len()],
        saved_args: Vec::new(),
        option_args: Vec::<(u16, u16)>::new(),
        arg_ranges: Vec::<Range<u16>>::new(),
        path_params: 0,
    };
    // Since the first arg, the URI host, is always
    // skipped we don't need to match on it
    let mut tree_index = 1;
    // let mut terminated = false;
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
                    + router.tree[c.selected as usize].child_span
                    + 1
            {
                if router.names
                    [router.segments[tree_index as usize].name as usize]
                    .starts_with(':')
                    || arg
                        == router.names[router.segments
                            [tree_index as usize]
                            .name
                            as usize]
                {
                    c.selected = tree_index;
                    tree_index += 1;
                    start = index + 1;
                    break;
                }
                // Skip to next sibling segment
                tree_index +=
                    router.tree[tree_index as usize].child_span + 1
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
                    if let Some(o) =
                        router.options.iter().position(|mapper| {
                            router.names[mapper.name as usize] == opt
                        })
                    {
                        c.option_occurrences[o] += 1;
                    } else {
                        // Not found
                    }
                    equal_sign = 0;
                } else {
                    if let Some(o) =
                        router.options.iter().position(|mapper| {
                            router.names[mapper.name as usize]
                                == &route[start..]
                        })
                    {
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
            if let Some(o) = router.options.iter().position(|mapper| {
                router.names[mapper.name as usize] == opt
            }) {
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
    Ok(c)
}

/// Parse a URI *up to* the Authority component, leaving
/// everything after it unparsed
pub fn parse_uri_scheme_and_authority(
    source: &[u8],
) -> URIAuthorityLengths {
    let mut lengths = URIAuthorityLengths {
        scheme_len: 0,
        user_len: 0,
        host_len: 0,
        port_len: 0,
    };
    let mut state = URIState::Scheme;
    let mut i = 0;
    let mut start = 0;
    while i < source.len() {
        match state {
            URIState::Scheme => match source[i] {
                b':' if i + 2 < source.len() => {
                    lengths.scheme_len = i as u8;
                    state = URIState::User;
                    if source[i + 1] == b'/' {
                        if source[i + 2] == b'/' {
                            if source[i + 3] == b'[' {
                                state = URIState::IPv6;
                            }
                            // Skip the "//"
                            start = i + 3;
                            i = start;
                            continue;
                        } else {
                            // ! Invalid, needs 2 '/'
                        }
                    } else {
                        start = i + 1;
                    }
                }
                b'/' => {
                    // Had an absolute or relative file path
                    return lengths;
                }
                _ => (),
            },
            URIState::IPv6 => {
                if source[i] == b']' {
                    state = URIState::Host;
                }
            }
            URIState::User => match source[i] {
                b'@' => {
                    lengths.user_len = (i - start) as u8;
                    state = URIState::Host;
                    start = i + 1;
                }
                b':' => {
                    // Didn't have a user, should become `host`
                    // Technically it could be a password, but
                    // that's not supported
                    lengths.host_len = (i - start) as u8;
                    state = URIState::Port;
                    start = i + 1;
                }
                b'/' => {
                    // Didn't have a user, should become `host`
                    lengths.host_len = (i - start) as u8;
                    // component = URIComponent::Path;
                    // start = i + 1;
                    return lengths;
                }
                _ => {}
            },
            URIState::Host => match source[i] {
                b':' => {
                    lengths.host_len = (i - start) as u8;
                    state = URIState::Port;
                    start = i + 1;
                }
                b'/' => {
                    lengths.host_len = (i - start) as u8;
                    // component = URIComponent::Path;
                    // start = i + 1;
                    return lengths;
                }
                _ => (),
            },
            URIState::Port => match source[i] {
                b'/' | b'?' | b'#' => {
                    lengths.port_len = (i - start) as u8;
                    // component = URIComponent::Path;
                    // start = i + 1;
                    return lengths;
                }
                b':' => {
                    lengths.host_len += (i - start) as u8;
                }
                _ => {}
            },
            // _ => (),
        }
        i += 1;
    }
    match (state, source[(lengths.scheme_len + 1) as usize] != b'/') {
        (URIState::User, true) => {
            lengths.host_len = 0;
        }
        (URIState::Port, true) => {
            lengths.host_len = 0;
        }
        (URIState::Port, _) => {
            lengths.port_len = (source.len() - start) as u8;
        }
        (URIState::User | URIState::Host, _) => {
            lengths.host_len = (source.len() - start) as u8;
        }
        _ => (),
    }
    lengths
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{data, Opt, OptArgKind, Segment, TreeNode};

    #[test]
    fn should_parse_uri_route_placeholder_segments() {
        // enum O {
        //     KeyOnly,
        //     Single1,
        //     Multi1,
        // }
        let router = Router {
            tree: &[
                // 0: https://example.com:443
                TreeNode {
                    child_span: 5,
                    parent: 0,
                },
                // 1: program
                TreeNode {
                    child_span: 4,
                    parent: 0,
                },
                // 2:   built-in
                TreeNode {
                    child_span: 1,
                    parent: 1,
                },
                // 3:     fun
                TreeNode {
                    child_span: 0,
                    parent: 2,
                },
                // 4:   :extension
                TreeNode {
                    child_span: 1,
                    parent: 1,
                },
                // 5:     fun
                TreeNode {
                    child_span: 0,
                    parent: 4,
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
                    opt_groups: 0,
                    name: 5,
                },
                Segment {
                    operands: 0,
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
                    name: 6,
                },
            ],
            actions: &[
                |_| Ok(()),
                |_| Ok(println!("program help")),
                |_| Ok(println!("built-in help")),
                |_| Ok(println!("fun help")),
                |_| Ok(println!("extension help")),
                |_| Ok(println!("fun help")),
            ],
            options: &[
                Opt {
                    kind: OptArgKind::KeyOnly,
                    name: 0,
                },
                Opt {
                    kind: OptArgKind::Single,
                    name: 1,
                },
                Opt {
                    kind: OptArgKind::Multiple,
                    name: 2,
                },
            ],
            short_option_mappers: &[(0, 'a'), (1, 'b'), (2, 'c')],
            names: &[
                "key-only",
                "single1",
                "multi1",
                "https://example.com:443",
                "program",
                "built-in",
                "fun",
                ":extension",
                /* "other-built-in", */
            ],
            summaries: &[
                "key-only summary",
                "single1 summary",
                "multi1 summary",
                "root summary",
                "program summary",
                "built-in summary",
                "fun summary",
                "extension summary",
                "extension fun summary",
                // "other-built-in summary",
            ],
            opt_group_rules: &[],
            opt_groups: &[],
            help_opt_index: None,
            doc: None,
        };
        let c = parse_uri_route(
            &router,
            "https://example.com:443/program/built-in/fun",
        )
        .unwrap();
        assert_eq!(c.selected, 3);

        let c = parse_uri_route(
            &router,
            "https://example.com:443/program/extensionA/fun",
        )
        .unwrap();
        assert_eq!(c.selected, 5);
    }
    #[test]
    fn should_parse_a_url_route_with_no_options_or_fragment() {
        let router = data();

        // Yes path, no options, no fragment
        let c =
            parse_uri_route(&router, "https://example.com:443/path/a/a2")
                .unwrap();
        assert_eq!(c.selected, 4);

        // No path, no options, no fragment
        let c =
            parse_uri_route(&router, "https://example.com:443").unwrap();
        assert_eq!(c.selected, 0);

        // Doesn't exist, `index` is the last recognized segment
        let c =
            parse_uri_route(&router, "https://example.com:443/path/b/b3")
                .unwrap();
        assert_eq!(c.selected, 5);
    }
    #[test]
    fn should_parse_a_url_route_with_options() {
        let router = data();

        // Yes path, yes options, no fragment
        let c = parse_uri_route(
            &router,
            "https://example.com:443/path/a/a2?single1=abc&key-only",
        )
        .unwrap();
        assert_eq!(c.selected, 4);
        assert_eq!(c.option_occurrences, [1, 1, 0]);

        // No path, yes options, no fragment
        let c =
            parse_uri_route(&router, "https://example.com:443?single1=0")
                .unwrap();
        assert_eq!(c.selected, 0);
        assert_eq!(c.option_occurrences, [0, 1, 0]);
    }
    // TODO: Figure out how a fragment fits into this segments and options concept
    #[test]
    fn should_parse_a_url_route_with_a_fragment() {
        let router = data();

        // URL route with no path, no options, yes fragment
        let c = parse_uri_route(&router, "https://example.com:443?#frag")
            .unwrap();
        assert_eq!(c.selected, 0);
        assert_eq!(c.option_occurrences, [0, 0, 0]);

        // URL route with no path, yes options, yes fragment
        let c = parse_uri_route(
            &router,
            "https://example.com:443?key-only#frag",
        )
        .unwrap();
        assert_eq!(c.selected, 0);
        assert_eq!(c.option_occurrences, [1, 0, 0]);

        // URL route with yes path, no options, yes fragment
        let c = parse_uri_route(
            &router,
            "https://example.com:443/path/a/a2#frag",
        )
        .unwrap();
        assert_eq!(c.selected, 4);
        assert_eq!(c.option_occurrences, [0, 0, 0]);

        // URL route with yes path, yes options, yes fragment
        let c = parse_uri_route(
            &router,
            "https://example.com:443/path/a/a2?key-only#frag",
        )
        .unwrap();
        assert_eq!(c.selected, 4);
        assert_eq!(c.option_occurrences, [1, 0, 0]);
    }
    #[test]
    fn should_parse_a_uri_string() {
        let mut lengths = parse_uri_scheme_and_authority(
      b"https://john.doe@www.example.com:123/forum/questions/?tag=networking&order=newest#top"
      );
        assert_eq!(lengths.scheme_len, 5);
        assert_eq!(lengths.user_len, 8);
        assert_eq!(lengths.host_len, 15);
        assert_eq!(lengths.port_len, 3);

        lengths = parse_uri_scheme_and_authority(
            b"ldap://[2001:db8::7]/c=GB?objectClass?one",
        );
        assert_eq!(lengths.scheme_len, 4);
        assert_eq!(lengths.user_len, 0);
        assert_eq!(lengths.host_len, 13);
        assert_eq!(lengths.port_len, 0);

        lengths =
            parse_uri_scheme_and_authority(b"mailto:John.Doe@example.com");
        assert_eq!(lengths.scheme_len, 6);
        assert_eq!(lengths.user_len, 8);
        assert_eq!(lengths.host_len, 11);
        assert_eq!(lengths.port_len, 0);

        lengths = parse_uri_scheme_and_authority(
            b"news:comp.infosystems.www.servers.unix",
        );
        assert_eq!(lengths.scheme_len, 4);
        assert_eq!(lengths.user_len, 0);
        assert_eq!(lengths.host_len, 0);
        assert_eq!(lengths.port_len, 0);

        lengths = parse_uri_scheme_and_authority(b"tel:+1-816-555-1212");
        assert_eq!(lengths.scheme_len, 3);
        assert_eq!(lengths.user_len, 0);
        assert_eq!(lengths.host_len, 0);
        assert_eq!(lengths.port_len, 0);

        lengths =
            parse_uri_scheme_and_authority(b"telnet://192.0.2.16:80/");
        assert_eq!(lengths.scheme_len, 6);
        assert_eq!(lengths.user_len, 0);
        assert_eq!(lengths.host_len, 10);
        assert_eq!(lengths.port_len, 2);

        lengths = parse_uri_scheme_and_authority(
            b"urn:oasis:names:specification:docbook:dtd:xml:4.1.2",
        );
        assert_eq!(lengths.scheme_len, 3);
        assert_eq!(lengths.user_len, 0);
        assert_eq!(lengths.host_len, 0);
        assert_eq!(lengths.port_len, 0);

        lengths = parse_uri_scheme_and_authority(
          b"http://[FEDC:BA98:7654:3210:FEDC:BA98:7654:3210]:80/index.html",
      );
        assert_eq!(lengths.scheme_len, 4);
        assert_eq!(lengths.user_len, 0);
        assert_eq!(lengths.host_len, 41);
        assert_eq!(lengths.port_len, 2);

        lengths = parse_uri_scheme_and_authority(
            b"http://[1080:0:0:0:8:800:200C:417A]/index.html",
        );
        assert_eq!(lengths.scheme_len, 4);
        assert_eq!(lengths.user_len, 0);
        assert_eq!(lengths.host_len, 28);
        assert_eq!(lengths.port_len, 0);

        lengths = parse_uri_scheme_and_authority(
            b"https://example.com:443?option1=abc",
        );
        assert_eq!(lengths.scheme_len, 5);
        assert_eq!(lengths.user_len, 0);
        assert_eq!(lengths.host_len, 11);
        assert_eq!(lengths.port_len, 3);

        lengths = parse_uri_scheme_and_authority(
            b"https://example.com:443#frag",
        );
        assert_eq!(lengths.scheme_len, 5);
        assert_eq!(lengths.user_len, 0);
        assert_eq!(lengths.host_len, 11);
        assert_eq!(lengths.port_len, 3);
    }
}

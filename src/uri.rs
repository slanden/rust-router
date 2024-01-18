enum URIState {
    Scheme,
    User,
    IPv6,
    Host,
    Port,
    Path,
    Query,
    Fragment,
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

/// Parse a URI *up to* the Authority component, leaving
/// everything after it unparsed
pub fn parse_uri_scheme_and_authority(source: &[u8]) -> URIAuthorityLengths {
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
            _ => (),
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

    #[test]
    fn should_parse_a_uri_string() {
        let mut lengths = parse_uri_scheme_and_authority(
        b"https://john.doe@www.example.com:123/forum/questions/?tag=networking&order=newest#top"
        );
        assert_eq!(lengths.scheme_len, 5);
        assert_eq!(lengths.user_len, 8);
        assert_eq!(lengths.host_len, 15);
        assert_eq!(lengths.port_len, 3);

        lengths = parse_uri_scheme_and_authority(b"ldap://[2001:db8::7]/c=GB?objectClass?one");
        assert_eq!(lengths.scheme_len, 4);
        assert_eq!(lengths.user_len, 0);
        assert_eq!(lengths.host_len, 13);
        assert_eq!(lengths.port_len, 0);

        lengths = parse_uri_scheme_and_authority(b"mailto:John.Doe@example.com");
        assert_eq!(lengths.scheme_len, 6);
        assert_eq!(lengths.user_len, 8);
        assert_eq!(lengths.host_len, 11);
        assert_eq!(lengths.port_len, 0);

        lengths = parse_uri_scheme_and_authority(b"news:comp.infosystems.www.servers.unix");
        assert_eq!(lengths.scheme_len, 4);
        assert_eq!(lengths.user_len, 0);
        assert_eq!(lengths.host_len, 0);
        assert_eq!(lengths.port_len, 0);

        lengths = parse_uri_scheme_and_authority(b"tel:+1-816-555-1212");
        assert_eq!(lengths.scheme_len, 3);
        assert_eq!(lengths.user_len, 0);
        assert_eq!(lengths.host_len, 0);
        assert_eq!(lengths.port_len, 0);

        lengths = parse_uri_scheme_and_authority(b"telnet://192.0.2.16:80/");
        assert_eq!(lengths.scheme_len, 6);
        assert_eq!(lengths.user_len, 0);
        assert_eq!(lengths.host_len, 10);
        assert_eq!(lengths.port_len, 2);

        lengths =
            parse_uri_scheme_and_authority(b"urn:oasis:names:specification:docbook:dtd:xml:4.1.2");
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

        lengths = parse_uri_scheme_and_authority(b"http://[1080:0:0:0:8:800:200C:417A]/index.html");
        assert_eq!(lengths.scheme_len, 4);
        assert_eq!(lengths.user_len, 0);
        assert_eq!(lengths.host_len, 28);
        assert_eq!(lengths.port_len, 0);

        lengths = parse_uri_scheme_and_authority(b"https://example.com:443?option1=abc");
        assert_eq!(lengths.scheme_len, 5);
        assert_eq!(lengths.user_len, 0);
        assert_eq!(lengths.host_len, 11);
        assert_eq!(lengths.port_len, 3);

        lengths = parse_uri_scheme_and_authority(b"https://example.com:443#frag");
        assert_eq!(lengths.scheme_len, 5);
        assert_eq!(lengths.user_len, 0);
        assert_eq!(lengths.host_len, 11);
        assert_eq!(lengths.port_len, 3);
    }
}

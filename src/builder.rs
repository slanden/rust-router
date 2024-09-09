use {
    crate::{Action, Context, OptGroupRules, Segment, TreeNode},
    std::{io, mem::transmute},
};

/// Temporary struct used when flattening a Seg tree
/// to be able to navigate out from a deep nest level
#[derive(Clone, Copy)]
struct Breadcrumb<'a> {
    seg: Seg<'a>,
    // The current child index in `seg`'s children
    child_index: usize,
    // The index in the output array
    final_index: usize,
}

#[derive(Clone, Copy)]
/// Allows declaring which options a `Cmd` expects.
pub struct OptGroup {
    options: &'static [u16],
    rules: u8,
}
impl OptGroup {
    /// Any of the options in this group can be present.  
    pub const fn anyof(options: &'static [impl Into<u16>]) -> Self {
        Self {
            options: unsafe { transmute(options) },
            rules: OptGroupRules::AnyOf as u8,
        }
    }
    /// Options in this group are exclusive; Only one of them
    /// can be present.
    pub const fn oneof(options: &'static [impl Into<u16>]) -> Self {
        Self {
            options: unsafe { transmute(options) },
            rules: OptGroupRules::OneOf as u8,
        }
    }
    /// Require at least one of the options in this group
    pub const fn required(mut self) -> Self {
        self.rules |= OptGroupRules::Required as u8;
        self
    }
}

// const fn validate_name(name: &str) {
//     #[cfg(feature = "posix-1")]
//     if self.name.len() < 2 || self.name.len() > 9 {
//         panic!("`name` must be 2-9 characters long.");
//     }

//     // #[cfg(feature = "posix-2")]
//     // if name
//     //     .chars()
//     //     .any(|c| c.is_uppercase() || !c.is_alphanumeric())
//     // {
//     //     // panic!("The name \"{}\" doesn't meet POSIX Guideline 2: Names must be all lowercase letters and digits from the portable character set.", name);
//     //     panic!()
//     // }
// }
// ! Compiles but doesn't work
// pub const fn strcmp(s: &'static str, _t: &'static str) -> bool {
//     match s {
//         _t => true,
//         _ => false,
//     }
// }

/// Represents a segment of your API that can be an action
/// or a nesting structure to lead to other segments. Think
/// URLs.
#[derive(Clone, Copy)]
pub struct Seg<'a> {
    name: &'static str,
    // summary: &'static str,
    commands: &'a [Seg<'a>],
    opt_groups: &'a [OptGroup],
    action: Action,
    // doc: DocGen,
    operands: u16,
}
impl<'a> Seg<'a> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            // summary,
            commands: &[],
            opt_groups: &[],
            action: default_action,
            // doc: doc::empty_doc,
            // sub_count: 0,
            operands: 0,
        }
        // let mut count = 0;
        // let mut index = 0;
        // const CN: usize = children.len();

        // while index < children.len() {
        //     count += children[index].sub_count + 1;
        //     index += 1;
        // }
        // let subs = &[Seg {
        //     name: "",
        //     sub_count: 0,
        //     commands: &[],
        // }; count];
        // Self {
        //     name,
        //     commands: &[],
        //     sub_count: count,
        // }
    }
    pub const fn action(mut self, f: Action) -> Self {
        self.action = f;
        self
    }
    /// Counts all commands in the tree, as well as their `OptGroup`s.
    pub const fn count<const DEPTH: usize>(&self) -> (usize, usize) {
        // Caches a parent and the selected child index to enable
        // depth-first search
        let mut breadcrumbs = [(
            &Seg {
                name: "",
                // summary: "",
                commands: &[],
                opt_groups: &[],
                action: default_action,
                // doc: doc::empty_doc,
                operands: 0,
            },
            0,
        ); DEPTH];
        let mut depth = 0;
        let mut count = 0;
        let mut groups = 0;
        count += 1;
        groups += self.opt_groups.len();
        breadcrumbs[0] = (self, 0);
        loop {
            if breadcrumbs[depth].1 < breadcrumbs[depth].0.commands.len() {
                let child =
                    &breadcrumbs[depth].0.commands[breadcrumbs[depth].1];
                count += 1;
                groups += child.opt_groups.len();
                breadcrumbs[depth].1 += 1;

                if !child.commands.is_empty() {
                    depth += 1;
                    breadcrumbs[depth].0 = &child;
                }
                continue;
            }
            if depth == 0 {
                break;
            }
            depth -= 1;
        }
        (count, groups)
    }
    // pub const fn doc(mut self, gen_fn: DocGen) -> Self {
    //     self.doc = gen_fn;
    //     self
    // }
    pub const fn flatten<
        const COUNT: usize,
        const GROUP_COUNT: usize,
        const STR_LIST_COUNT: usize,
    >(
        self,
        opt_names: &[&'static str],
    ) -> (
        [TreeNode; COUNT],
        [Segment; COUNT],
        [Action; COUNT],
        // [DocGen; COUNT],
        [u8; GROUP_COUNT],
        [&'static [u16]; GROUP_COUNT],
        [&'static str; STR_LIST_COUNT],
        // [&'static str; STR_LIST_COUNT],
    ) {
        let mut tree = [TreeNode {
            child_span: 0,
            parent: 0,
        }; COUNT];
        let mut segments = [Segment {
            operands: 0,
            name: 0,
            opt_groups: 0,
        }; COUNT];
        let mut actions: [Action; COUNT] = [default_action; COUNT];
        // let mut doc_gens: [DocGen; COUNT] = [doc::empty_doc; COUNT];
        let mut opt_grp_rules: [u8; GROUP_COUNT] = [0; GROUP_COUNT];
        let mut opt_grps: [&[u16]; GROUP_COUNT] = [&[]; GROUP_COUNT];
        // Potentially more space than needed
        let mut names = [""; STR_LIST_COUNT];
        // let mut summaries = [""; STR_LIST_COUNT];
        // Facilitates a depth-first search
        let mut breadcrumbs = [Breadcrumb {
            seg: Seg {
                name: "",
                // summary: "",
                commands: &[],
                opt_groups: &[],
                action: default_action,
                // doc: doc::empty_doc,
                operands: 0,
            },
            child_index: 0,
            final_index: 0,
        }; COUNT];
        let mut depth = 0;
        let mut count = 0;
        let mut opt_group_index = 0;
        let mut child_opt_group_index = 0;

        while count < opt_names.len() {
            names[count] = opt_names[count];
            // summaries[count] = opt_summaries[count];
            count += 1;
        }
        count = 0;

        names[0 + opt_names.len()] = self.name;
        segments[0].name = opt_names.len() as u16;
        // summaries[0 + opt_names.len()] = self.summary;
        actions[0] = self.action;
        // doc_gens[0] = self.doc;
        if !self.opt_groups.is_empty() {
            segments[0].opt_groups = (self.opt_groups.len() as u16) << 12
                | opt_group_index as u16;
            while child_opt_group_index < self.opt_groups.len() {
                opt_grps[opt_group_index] =
                    self.opt_groups[child_opt_group_index].options;
                opt_grp_rules[opt_group_index] =
                    self.opt_groups[child_opt_group_index].rules;
                child_opt_group_index += 1;
                opt_group_index += 1;
            }
            child_opt_group_index = 0;
        }

        if self.operands > 0 && self.commands.len() > 0 {
            // TODO: Figure out a way to error here; segments with children should not expect operands
            segments[0].operands = 0;
        } else {
            segments[0].operands = self.operands;
        }
        count += 1;
        breadcrumbs[0].seg = self;
        breadcrumbs[0].final_index = 0;

        loop {
            if breadcrumbs[depth].child_index
                < breadcrumbs[depth].seg.commands.len()
            {
                let child = breadcrumbs[depth].seg.commands
                    [breadcrumbs[depth].child_index];

                // TODO: Check name for uniqueness when
                //       strings can be compared in const fn

                names[count + opt_names.len()] = child.name;
                segments[count].name = (count + opt_names.len()) as u16;
                // summaries[count + opt_names.len()] = child.summary;
                actions[count] = child.action;
                // doc_gens[count] = child.doc;
                if !child.opt_groups.is_empty() {
                    segments[count].opt_groups =
                        (child.opt_groups.len() as u16) << 12
                            | opt_group_index as u16;
                    while child_opt_group_index < child.opt_groups.len() {
                        opt_grps[opt_group_index] = child.opt_groups
                            [child_opt_group_index]
                            .options;
                        opt_grp_rules[opt_group_index] =
                            child.opt_groups[child_opt_group_index].rules;
                        child_opt_group_index += 1;
                        opt_group_index += 1;
                    }
                    child_opt_group_index = 0;
                }
                segments[count].operands = child.operands;
                if child.operands > 0 && child.commands.len() > 0 {
                    // TODO: Figure out a way to error here
                    segments[count].operands = 0;
                } else {
                    segments[count].operands = child.operands;
                }
                count += 1;
                breadcrumbs[depth].child_index += 1;
                tree[count - 1].parent =
                    breadcrumbs[depth].final_index as u16;

                if !child.commands.is_empty() {
                    depth += 1;
                    breadcrumbs[depth].seg = child;
                    breadcrumbs[depth].final_index = count - 1;
                }
                continue;
            }

            // When ascending one level of depth, use the current
            // count to get how many items were added since then,
            // regardless of how many levels deep, then subtract
            // one to exclude the current item
            tree[breadcrumbs[depth].final_index].child_span +=
                (count - breadcrumbs[depth].final_index - 1) as u16;

            if depth == 0 {
                break;
            }
            depth -= 1;
        }
        (
            tree,
            segments,
            actions,
            // doc_gens,
            opt_grp_rules,
            opt_grps,
            names,
            // summaries,
        )
    }
    pub const fn nest(mut self, commands: &'a [Seg]) -> Self {
        self.commands = commands;
        self
    }
    pub const fn operands(mut self, operands: u16) -> Self {
        self.operands = operands;
        self
    }
    pub const fn options(mut self, groups: &'a [OptGroup]) -> Self {
        self.opt_groups = groups;
        self
    }
}

pub fn default_action(_: Context) -> io::Result<()> {
    Ok(())
}

/// Creates a `Router` from a `Seg` tree.
///
/// Param1: The *enum* that defines the options.
///
/// Param2: A `const` variable the `Seg` is assigned to.
///
/// ## Example
/// ```ignore
/// optmap!(enum O using []);
///
/// fn main() {
///   const SEG: Seg<O> = Seg::new("example", "An example");
///
///   router!(O, SEG);
/// }
/// ```
#[macro_export]
macro_rules! router {
    ($opt_enum: ident, $seg: ident) => {{
        // Returns list of options, option mappers, and
        // *the* names array the other names array will
        // be appended to by commands
        const _OPS: (
            &[router::Opt],
            &[(u16, char)],
            &[&str],
            &[&str],
            Option<u16>,
        ) = $opt_enum::list();
        const _CMD_COUNT: (usize, usize) = $seg.count::<16>();
        const _STR_COUNT: usize = _CMD_COUNT.0 + _OPS.2.len();
        const _CMD_PARTS: (
            [router::TreeNode; _CMD_COUNT.0],
            [router::Segment; _CMD_COUNT.0],
            [router::Action; _CMD_COUNT.0],
            // [router::DocGen; _CMD_COUNT.0],
            [u8; _CMD_COUNT.1],
            [&[u16]; _CMD_COUNT.1],
            [&str; _STR_COUNT],
            // [&str; _STR_COUNT],
        ) = $seg
            .flatten::<{ _CMD_COUNT.0 }, { _CMD_COUNT.1 }, _STR_COUNT>(
                _OPS.2,
            );

        // ? For some reason, creating the router struct through this
        // ? function instead of directly uses ~41 more bytes. But,
        // ? creating directly means exposing private fields
        Router::from_raw_parts(
            &_CMD_PARTS.0,
            &_CMD_PARTS.1,
            &_CMD_PARTS.2,
            // docs: &_CMD_PARTS.3,
            &_CMD_PARTS.3,
            &_CMD_PARTS.4,
            &_CMD_PARTS.5,
            // summaries: &_CMD_PARTS.7,
            _OPS.0,
            _OPS.1,
            _OPS.4,
        )
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Clone, Copy)]
    enum O {
        OptionA,
        OptionB,
        OptionC,
    }
    impl Into<u16> for O {
        fn into(self) -> u16 {
            self as u16
        }
    }

    #[test]
    fn should_count_all_tree_segments_and_their_opt_groups() {
        const TEST: Seg = Seg {
            name: "test",
            // summary: "",
            commands: &[],
            opt_groups: &[],
            action: |_| Ok(()),
            // doc: doc::empty_doc,
            operands: 0,
        };
        const CONFIG: Seg = Seg {
            name: "config",
            // summary: "",
            commands: &[
                Seg {
                    name: "command",
                    // summary: "",
                    commands: &[
                        Seg {
                            name: "deep1",
                            // summary: "",
                            commands: &[],
                            opt_groups: &[
                                OptGroup {
                                    options: &[O::OptionA as u16],
                                    rules: OptGroupRules::AnyOf as u8,
                                },
                                OptGroup {
                                    options: &[
                                        O::OptionB as u16,
                                        O::OptionC as u16,
                                    ],
                                    rules: OptGroupRules::OneOf as u8
                                        | OptGroupRules::Required as u8,
                                },
                            ],
                            action: |_| Ok(()),
                            // doc: doc::empty_doc,
                            operands: 0,
                        },
                        Seg {
                            name: "deep2",
                            // summary: "",
                            commands: &[],
                            opt_groups: &[OptGroup {
                                options: &[
                                    O::OptionA as u16,
                                    O::OptionB as u16,
                                ],
                                rules: OptGroupRules::AnyOf as u8,
                            }],
                            action: |_| Ok(()),
                            // doc: doc::empty_doc,
                            operands: 0,
                        },
                    ],
                    opt_groups: &[],
                    action: |_| Ok(()),
                    // doc: doc::empty_doc,
                    operands: 0,
                },
                Seg {
                    name: "action",
                    // summary: "",
                    commands: &[],
                    opt_groups: &[],
                    action: |_| Ok(()),
                    // doc: doc::empty_doc,
                    operands: 0,
                },
            ],
            opt_groups: &[],
            action: |_| Ok(()),
            // doc: doc::empty_doc,
            operands: 0,
        };
        let (size, groups) = TEST
            .nest(&[
                CONFIG,
                Seg {
                    name: "add",
                    // summary: "",
                    commands: &[],
                    opt_groups: &[OptGroup {
                        options: &[O::OptionA as u16, O::OptionC as u16],
                        rules: OptGroupRules::AnyOf as u8
                            | OptGroupRules::Required as u8,
                    }],
                    action: |_| Ok(()),
                    // doc: doc::empty_doc,
                    operands: 0,
                },
            ])
            .count::<16>();
        assert_eq!(size, 7);
        assert_eq!(groups, 4);
    }
    #[test]
    fn should_encode_a_tree_of_segments_into_a_flat_array() {
        const TEST: Seg = Seg {
            name: "test",
            // summary: "",
            commands: &[],
            opt_groups: &[],
            action: |_| Ok(()),
            // doc: doc::empty_doc,
            operands: 0,
        };
        const CONFIG: Seg = Seg {
            name: "config",
            // summary: "",
            commands: &[
                Seg {
                    name: "command",
                    // summary: "",
                    commands: &[
                        Seg {
                            name: "deep1",
                            // summary: "",
                            commands: &[],
                            opt_groups: &[
                                OptGroup {
                                    options: &[O::OptionA as u16],
                                    rules: OptGroupRules::AnyOf as u8,
                                },
                                OptGroup {
                                    options: &[
                                        O::OptionB as u16,
                                        O::OptionC as u16,
                                    ],
                                    rules: OptGroupRules::OneOf as u8
                                        | OptGroupRules::Required as u8,
                                },
                            ],
                            action: |_| Ok(()),
                            // doc: doc::empty_doc,
                            operands: 0,
                        },
                        Seg {
                            name: "deep2",
                            // summary: "",
                            commands: &[],
                            opt_groups: &[OptGroup {
                                options: &[
                                    O::OptionA as u16,
                                    O::OptionB as u16,
                                ],
                                rules: OptGroupRules::AnyOf as u8,
                            }],
                            action: |_| Ok(()),
                            // doc: doc::empty_doc,
                            operands: 0,
                        },
                    ],
                    opt_groups: &[],
                    action: |_| Ok(()),
                    // doc: doc::empty_doc,
                    operands: 0,
                },
                Seg {
                    name: "action",
                    // summary: "",
                    commands: &[],
                    opt_groups: &[],
                    action: |_| Ok(()),
                    // doc: doc::empty_doc,
                    operands: 0,
                },
            ],
            opt_groups: &[],
            action: |_| Ok(()),
            // doc: doc::empty_doc,
            operands: 0,
        };
        const FLATTENED_FROM_STRUCTS: (
            [TreeNode; 7],
            [Segment; 7],
            [Action; 7],
            // [DocGen; 7],
            [u8; 4],
            [&[u16]; 4],
            [&str; 7],
            // [&str; 7],
        ) = TEST
            .nest(&[
                CONFIG,
                Seg {
                    name: "add",
                    // summary: "",
                    commands: &[],
                    opt_groups: &[OptGroup {
                        options: &[O::OptionA as u16, O::OptionC as u16],
                        rules: OptGroupRules::AnyOf as u8
                            | OptGroupRules::Required as u8,
                    }],
                    action: |_| Ok(()),
                    // doc: doc::empty_doc,
                    operands: 0,
                },
            ])
            .flatten::<7, 4, 7>(&[]);

        let expected = (
            [
                TreeNode {
                    child_span: 6,
                    parent: 0,
                },
                TreeNode {
                    child_span: 4,
                    parent: 0,
                },
                TreeNode {
                    child_span: 2,
                    parent: 1,
                },
                TreeNode {
                    child_span: 0,
                    parent: 2,
                },
                TreeNode {
                    child_span: 0,
                    parent: 2,
                },
                TreeNode {
                    child_span: 0,
                    parent: 1,
                },
                TreeNode {
                    child_span: 0,
                    parent: 0,
                },
            ],
            [
                Segment {
                    name: 0,
                    operands: 0,
                    opt_groups: 0,
                },
                Segment {
                    name: 1,
                    operands: 0,
                    opt_groups: 0,
                },
                Segment {
                    name: 2,
                    operands: 0,
                    opt_groups: 0,
                },
                Segment {
                    name: 3,
                    operands: 0,
                    opt_groups: 2 << 12,
                },
                Segment {
                    name: 4,
                    operands: 0,
                    opt_groups: 1 << 12 | 2,
                },
                Segment {
                    name: 5,
                    operands: 0,
                    opt_groups: 0,
                },
                Segment {
                    name: 6,
                    operands: 0,
                    opt_groups: 1 << 12 | 3,
                },
            ],
        );

        assert_eq!(FLATTENED_FROM_STRUCTS.0.len(), expected.0.len());

        const FLATTENED_FROM_BUILDER: (
            [TreeNode; 7],
            [Segment; 7],
            [Action; 7],
            // [DocGen; 7],
            [u8; 4],
            [&[u16]; 4],
            [&str; 7],
            // [&str; 7],
        ) = Seg/* ::<O> */::new("test")
            .nest(&[
                Seg::new("config").nest(&[
                    Seg::new("command").nest(&[
                        Seg::new("deep1").options(&[
                            OptGroup {
                                options: &[O::OptionA as u16],
                                rules: OptGroupRules::AnyOf as u8,
                            },
                            OptGroup {
                                options: &[
                                    O::OptionB as u16,
                                    O::OptionC as u16,
                                ],
                                rules: OptGroupRules::OneOf as u8
                                    | OptGroupRules::Required as u8,
                            },
                        ]),
                        Seg::new("deep2").options(&[OptGroup {
                            options: &[
                                O::OptionA as u16,
                                O::OptionB as u16,
                            ],
                            rules: OptGroupRules::AnyOf as u8,
                        }]),
                    ]),
                    Seg::new("action"),
                ]),
                Seg::new("add").options(&[OptGroup {
                    options: &[O::OptionA as u16, O::OptionC as u16],
                    rules: OptGroupRules::AnyOf as u8
                        | OptGroupRules::Required as u8,
                }]),
            ])
            .flatten::<7, 4, 7>(&[]);
        assert_eq!(FLATTENED_FROM_BUILDER.0.len(), expected.0.len());

        for i in 0..FLATTENED_FROM_STRUCTS.0.len() {
            assert_eq!(
                FLATTENED_FROM_STRUCTS.1[i].name,
                expected.1[i].name
            );
            assert_eq!(
                FLATTENED_FROM_STRUCTS.0[i].child_span,
                expected.0[i].child_span
            );
            assert_eq!(
                FLATTENED_FROM_STRUCTS.0[i].parent,
                expected.0[i].parent
            );
            assert_eq!(
                FLATTENED_FROM_STRUCTS.1[i].opt_groups,
                expected.1[i].opt_groups,
                "i: {}, {}",
                i,
                FLATTENED_FROM_STRUCTS.1[i].opt_groups,
            );

            assert_eq!(
                FLATTENED_FROM_BUILDER.1[i].name,
                expected.1[i].name
            );
            assert_eq!(
                FLATTENED_FROM_BUILDER.0[i].child_span,
                expected.0[i].child_span
            );
            assert_eq!(
                FLATTENED_FROM_BUILDER.0[i].parent,
                expected.0[i].parent
            );
            assert_eq!(
                FLATTENED_FROM_BUILDER.1[i].opt_groups,
                expected.1[i].opt_groups
            );
        }

        let op_rules = [
            OptGroupRules::AnyOf as u8,
            OptGroupRules::OneOf as u8 | OptGroupRules::Required as u8,
            OptGroupRules::AnyOf as u8,
            OptGroupRules::AnyOf as u8 | OptGroupRules::Required as u8,
        ];
        assert_eq!(FLATTENED_FROM_STRUCTS.3, op_rules);
        assert_eq!(FLATTENED_FROM_BUILDER.3, op_rules);

        assert_eq!(FLATTENED_FROM_STRUCTS.4[0], &[O::OptionA as u16]);
        assert_eq!(
            FLATTENED_FROM_STRUCTS.4[1],
            &[O::OptionB as u16, O::OptionC as u16]
        );
        assert_eq!(
            FLATTENED_FROM_STRUCTS.4[2],
            &[O::OptionA as u16, O::OptionB as u16]
        );
        assert_eq!(
            FLATTENED_FROM_STRUCTS.4[3],
            &[O::OptionA as u16, O::OptionC as u16]
        );
    }
    #[test]
    fn should_set_segment_operands_to_zero_when_it_has_children() {
        let parts = Seg::new("path")
            .nest(&[Seg::new("a")
                .operands(1)
                .nest(&[Seg::new("a1"), Seg::new("a2")])])
            .flatten::<4, 0, 4>(&[]);
        assert_eq!(parts.1[1].operands, 0);
    }
}

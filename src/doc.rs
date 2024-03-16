use crate::Context;

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

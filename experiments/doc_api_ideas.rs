use router::*;
use std::fs;

optmap!(enum O using [
  Format | 'f',
  Help | 'h',
  Quiet,
  Req,
  ValueArg > String,
  Version,
]);

fn main_without_doc_method() {
    const C: Seg = Seg::new("example", "").nest(&[
        Seg::new("add", "")
            .action(|c| {
                if c.has_opt(O::Help) {
                    let docs = fs::read_to_string("path/to/docs")
                        .unwrap_or("No docs".to_string());

                    return Ok(println!("{}", docs));
                }

                Ok(println!("Add some things"))
            })
            .operands(2),
        Seg::new("divide", "").action(|c| {
            if c.has_opt(O::Help) {
                let docs = "Could be generated however you want";

                return Ok(println!("{}", docs));
            }

            Ok(println!("Divide two things."))
        }),
    ]);
    const r: Router = router!(O, C);
    r.run().unwrap();
}

// Complete docs are created and printed inside a
// `.doc(c: &Context)` method
fn main_with_doc_method_only() {
    const C: Seg = Seg::new("example", "").nest(&[
        Seg::new("add", "")
            .action(|_| Ok(println!("Add some things")))
            .operands(2)
            .doc(|_| {
                let complete_docs = fs::read_to_string("path/to/docs")
                    .unwrap_or("No docs".to_string());
            }),
        Seg::new("divide", "")
            .action(|_| Ok(println!("Divide two things.")))
            .doc(|_| {
                //
                default_doc_blocks(&c).summary("div summary!")
            }),
    ]);
    const r: Router = router!(O, C);
    r.run().unwrap();
}

// Potentially, only command-specific parts of the docs are
// created. As much as possible is created by a global
// handler, and the command-specific parts are merged in.
// Then, they are printed.
fn main_with_global_handler_and_doc_method() {
    const C: Seg = Seg::new("example", "").nest(&[
        Seg::new("add", "")
            .action(|_| Ok(println!("Add some things")))
            .operands(2)
            .doc(|_, _unused_blocks| {
                let complete_docs = fs::read_to_string("path/to/docs")
                    .unwrap_or("No docs".to_string());
            }),
        Seg::new("divide", "")
            .action(|_| Ok(println!("Divide two things.")))
            .doc(|_, blocks| blocks.summary("div summary!")),
    ]);

    const r: Router = router!(O, C);

    let c = r.context().unwrap();
    if c.has_opt(O::Help) {
        let docs = c.docs(&c, default_doc_blocks(&c));
        println!("{}", docs);
    } else {
        // Run the action
        c.action();
    }
}

// # Pros and Cons of how docs are stored
//
// Not having a `doc()` method. Not really the same
// type of comparison, but I'm not sure where else to
// list it.
// + Avoid storing additional callback for every command
//      Essentially trading the size of a callback for the
//      size of an `if` check
// - More boilerplate
// ```
// .action(|c| {
//   if c.has_opt(O::Help) {
//       default_doc_blocks(&c).summary("div summary!");
//       return Ok(());
//   }
//   Ok(println!("I'm printing"))
// })
// ```
// Reading from a file
// + Not in memory when not needed
// + Not in the binary
// - Wrking with the file system (permissions,
//   where to place, does it exist or not, etc.)
// - Managing different versions installed on the system
// ```
// .doc(|_, _unused_blocks| {
//   let docs = fs::read_to_string("path/to/docs")
//       .unwrap_or("No docs".to_string());
//     blocks.summary("")
//
//     println!("{}", docs);
// })
// ```
// Generating at runtime
// + Not in memory when not needed
// - Stored in the binary
// ```
// .doc(|_, blocks| {
//   blocks.summary("div summary!")
//
//   println!("{}", docs);
// })
// ```
// Storing docs like other `Router` parts
// (`&'static [&'static str]``)
// + Doc gen is a little more automatic
// - Little to no freedom in their doc gen
// - Always in memory
// - Stored in the binary
//
// Another con if docs are separated from the library
// - API complexity, i.e. would need to use the ugly const
//   functions hidden by the `router!()` macro
// ```
// .doc(|c, blocks| {
//   blocks.summary("")
//
//   println!("{}", global_docs[c.selected]);
// });
// ```
//
// # Another alternative
//
// Make `Seg` take a generic type argument for a Doc
// builder type to allow `.doc()` to take it as input,
// and the user has a global Help option handler and
// pass that type created with a default to the `.doc()`
// handler. The user might need to manually call the
// command's `action` if the Help option is not passed,
// but I'm not sure

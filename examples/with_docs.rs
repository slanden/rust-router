//! The documentation strategy is still a work-in-progress.
//! The API here is only one option not decided on yet.
//! See *./doc_api_ideas.rs*

use router::*;

optmap!(enum O using [
  Format | 'f',
  Help | 'h',
  Quiet,
  Req,
  ValueArg > String,
  Version,
]);

fn main() {
    const C: Seg = Seg::new("example", "example summary").nest(&[
        Seg::new("add", "add summary")
            .action(|_| Ok(println!("Add some things")))
            .operands(2),
        Seg::new("divide", "div summary")
            .action(|_| Ok(println!("Divide two things."))),
        // .doc(|_, blocks| blocks.summary("div summary!")),
        Seg::new("print", "print summary")
            .action(|_| Ok(println!("I'm printing"))),
        // .doc(|_, blocks| {
        //     let mut blocks = blocks.summary("Print dat summary");
        //     // blocks
        //     //     .options(O::ValueArg)
        //     //     .append(text("An example"))
        //     //     .insert_after(
        //     //         "Some Section",
        //     //         block("Another section", vec![text("Body text")]),
        //     //     );
        //     blocks.push(inline(vec![
        //         text("For example\n"),
        //         text("print -h").indent().bg_color(128, 128, 128),
        //     ]));
        //     // blocks.push(indent);
        //     blocks
        // }),
        // .doc(fnc),
    ]);
    let r = router!(O, C);
    r.run().unwrap();
}

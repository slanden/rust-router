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
        Seg::new("print", "print summary")
            .action(|_| Ok(println!("I'm printing"))),
        Seg::new("help", "Generate Manual pages")
            .operands(u16::MAX)
            .action(|c| {
                let command = c
                    .router
                    .parse(c.operands().into_iter().map(|s| s.to_owned()))?
                    .selected;
                Ok(println!(
                    "Generate man page for the command at index {}",
                    command
                ))
            }),
    ]);
    let r = router!(O, C);
    r.run().unwrap();
}

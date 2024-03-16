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
    ]);
    let r = router!(O, C, help);
    r.run().unwrap();
}
fn help(_: &Context) -> String {
    String::from("custom")
}

use router::*;

optmap!(enum O using [
  // Regular comment
  Format | 'f',
  Help | 'h',
  Quiet,
  Req,
  ValueArg > String,
  MultiArg > String[],
  /// Summary text for
  ///
  /// `Version`
  Version,
]);

fn main() {
    const C: Seg = Seg::new("example", "an example program").nest(&[
        Seg::new("add", "add numbers").action(|_| {
            //
            Ok(())
        }),
        Seg::new("divide", "div nums").operands(1).nest(&[Seg::new(
            "example",
            "div nums example",
        )
        .options(&[OptGroup::anyof(&[O::ValueArg])])
        .action(|c| {
            println!(
                "{:?}\n{:?}\n{:?}",
                c.option_args, c.saved_args, c.arg_ranges
            );
            Ok(())
        })]),
        Seg::new("print", "print some things")
            .operands(1)
            .action(|_| Ok(println!("I'm printing!"))),
    ]);
    const R: Router = router!(O, C);
    R.run().unwrap();
}

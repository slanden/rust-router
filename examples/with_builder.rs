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
    const C: Seg = Seg::new("example").nest(&[
        Seg::new("add").action(|_| {
            //
            Ok(())
        }),
        Seg::new("divide").operands(1).nest(&[Seg::new("example")
            .options(&[OptGroup::anyof(&[O::ValueArg])])
            .action(|c| {
                println!(
                    "{:?}\n{:?}\n{:?}",
                    c.option_args, c.saved_args, c.arg_ranges
                );
                Ok(())
            })]),
        Seg::new("print")
            .operands(1)
            .action(|_| Ok(println!("I'm printing!"))),
    ]);
    const R: Router = router!(O, C);
    R.run().unwrap();
}

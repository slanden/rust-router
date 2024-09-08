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
        Seg::new("add").action(|_| Ok(())).operands(u16::MAX),
        Seg::new("divide").nest(&[Seg::new("example")
        .options(&[OptGroup::anyof(&[O::ValueArg])])
        .action(|c| {
            router::context_size(&c);
            Ok(())
        })]),
        Seg::new("print")
            .operands(1)
            .action(|_| Ok(println!("I'm printing!"))),
    ]);
    const R: Router = router!(O, C);
    // println!(
    //   "opts: {:#?}\nshort opts: {:?}\nsegs: {:#?}\nactions: {:?}\nnames: {:?}",
    //   R.options,
    //   R.short_option_mappers,
    //   R.segments,
    //   R.actions,
    //   R.names
    // );
    R.run().unwrap();
}

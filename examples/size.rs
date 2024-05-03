
fn main() {
    const C: Seg = Seg::new("example", "an example program").nest(&[
        Seg::new("add", "add numbers")
            .action(|_| Ok(()))
            .operands(u16::MAX),
        Seg::new("divide", "div nums").nest(&[Seg::new(
            "example",
            "div nums example",
        )
        .options(&[OptGroup::anyof(&[O::ValueArg])])
        .action(|c| {
            router::context_size(&c);
            Ok(())
        })]),
        Seg::new("print", "print some things")
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

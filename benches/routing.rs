#![allow(non_camel_case_types)]
use {
    criterion::{criterion_group, criterion_main, Criterion},
    router::*,
    std::{ffi::OsString, path::PathBuf, str::FromStr},
};

#[allow(dead_code)]
#[derive(Debug)]
struct AppArgs {
    number: i32,
    opt_number: Option<u32>,
    width: u16,
    input: Vec<std::path::PathBuf>,
}

pub fn criterion_benchmark(c: &mut Criterion) {
    optmap!(enum Small_O using [
      Format | 'f',
      Help | 'h',
      Quiet,
      Req,
      ValueArg > String,
      MultiArg > String[],
      Number > i32,
      OptNumber > u32,
      Width > u16,
    ]);
    const SMALL_C: Seg =
        Seg::new("example", "an example program").nest(&[
            Seg::new("add", "add numbers").action(|_| {
                //
                Ok(())
            }),
            Seg::new("divide", "div nums")
                .operands(1)
                .nest(&[Seg::new("example", "div nums example")]),
            Seg::new("print", "print some things")
                .operands(1)
                .action(|_| Ok(println!("I'm printing!"))),
        ]);
    pub const SMALL_R: Router = router!(Small_O, SMALL_C);
    let small_args = vec![
        OsString::from_str("value-arg").unwrap(),
        OsString::from_str("1").unwrap(),
        OsString::from_str("--req").unwrap(),
        OsString::from_str("--version").unwrap(),
        OsString::from_str("--multi-arg").unwrap(),
        OsString::from_str("green").unwrap(),
        OsString::from_str("--value-arg").unwrap(),
        OsString::from_str("2").unwrap(),
        OsString::from_str("--opt-number").unwrap(),
        OsString::from_str("22").unwrap(),
        OsString::from_str("--multi-arg").unwrap(),
        OsString::from_str("purple").unwrap(),
        OsString::from_str("--width").unwrap(),
        OsString::from_str("4").unwrap(),
        OsString::from_str("--quiet").unwrap(),
        OsString::from_str("--number").unwrap(),
        OsString::from_str("10").unwrap(),
    ];

    optmap!(enum Large_O using [
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
        uybjuudw61nttsd,
        ntks4xuhmykh4rr,
        r9pkurgvexrchla,
        mm9nwrhko4ep0u5,
        fzxdwrhxplfs9di,
        dnmctdytqgmofrz,
        kroo3fc5niqx6be,
        kfodzvmsorcbkgo,
        p31pxrjwnymxwfr,
        pr6vjra6oaakimd > String[],
        lqlhwk41mtp4se0 > String[],
        rm4jpdgggguvxnn > String[],
        fzt9ov0ktohvcni > String[],
        pax7ppo1xblrbxv > String[],
        rijfrvzdpi4fdu7 > String[],
        w07dignsqcxfms0 > String[],
        uhnb9m3ctlpdvke > String[],
        ojlak58ay2zgnox,
        hxzjgpxqajdomkx,
        zoanpjknovk88q2,
        jwczs3zeuy8xnee,
        mhtjd5762st3yf7,
        cmjmlzmd2alxgny,
        gd3ubnxgld3gn8p,
        mxitjrm9x9sgu5w,
        llnvbr33cnvgwd0,
        md4ygsxxb1dny9w,
        s9f6j79dnb6wsgk,
        rl2x9ggyr1uqjlw,
        ycunkb0stxie2dt,
        yb5ghmlhkyro2rv,
        khtrli4xvxxviy6,
        den57vpd7mmggz9,
        nm9ny7ij1xagsxt,
        px8puhlpoeokuv2,
        bhtoxwkuuxeqx6s,
        mliopos2lgiawyk,
        r2rk5qkrcoqjm4s,
        cqsuaawab829b0h,
        fskfksrjtencsg9,
        es8rnkfp9al6xvu,
        vodaltbdm0md1rn,
        lztaebsppph4jzd,
        agmut3viqkh8p0r,
        bd9hz9e64ow7oen,
        u8meltydb3eyq4g,
        diuauaayab1aqqw,
        h1cpp68oasnhfvx,
        tmwzux561lrjrwb,
        aknxocehgif5snc,
        hr3fngobe0tyeo0,
        d9ws0zdsq7qazwi,
        gqxnkhov0aqf8px,
        ar6ty9pycantkzg,
        icqd78aed63fd52,
        p7qj6oaipie7r22,
        fzifwan0odeoohq,
        vdazscqinulx0h4,
        ixoxlqdw6ub8woh,
        zubn0lhzor3pcux,
        voaz8beqnjpdlci,
        eqoft2o2oxvach4,
        c8obeyhsn6hsxd7,
        opciztyvsztacx8,
        p2xbw5o80wi7ra9,
        k3jdyfcuptgp1le,
        xkxkyelvkj7vheg,
        e55etmkros82faq,
        ubir2vrv8pz3dxw,
        w5byydotq3szac7,
        d9thg7eanlkmmdv > String[],
        ub5iuatjgn5qw6f,
        h1q7fegeeoz13je,
        pai7r57crokbkha,
        p5ys0c8kahhfpi6,
        pofstu57e88ebfm,
        veaksu0omqqmumu,
        x14aqxb8sjjom62,
        fxaf9yte0vm9y99,
        ozste5e2sa5rmot,
        galp1cdd7ok3y87,
        kbhifnimwxn5srg,
        gbbtn9qye6vwwbw,
        wl64ztorzr5nx52,
        rae15xsyvfulsh6,
        kmtkssv5p7crqqu,
        gof68yw1p1v1f85,
        szb7xohk9cgleb1,
        bsvysf9anmfz36g,
        czgyizh8tfs9vv8,
        ndvghnt1qsiuvol,
        opii2i8zuoarbnf,
        oiwjtoecqsro7nc,
        zvj2dozikzurcpc,
        dxkvdjmwngsb0fi,
        klwd9u5ifdwiutt,
        wtdtop531fea7pk,
        aivubbaeygrmtfb,
        f38nk6tjqqsbmhw,
        km6dougc987k0k0,
        vpmtot59ugihrh0,
        ua3s0pnmm5qoqo3,
        soozcjjoei7cq5o,
        y9fjsr5yarhgeeh,
        uyea8cqkahzkamd,
        eckwou8xs9bsamd,
        lbspcsg4keo2cg8,
        cjmekpbos78afw1,
        vx7jqwzcxli1xcr,
        s2ti3zr4cepy6jn,
        dpnh0gsvwegghgg,
        zfodmqyolrp8cxn,
        edqcezrls504s1b,
        kxgnbffjrgmokkh,
        wxhynmzffypu6gq,
        wq63xtdi6slfbvi,
        q84jnst3kwhks4b,
        kr3mindmwnuoe32,
        qwkxw2iao50v4nl,
        kes6j1h5say2ad5,
        xgegrf2q3sdcr29,
        djvfleilppi1trd,
        npa1rqwlcnibvzm,
        miwnsl9mvdrqyyf,
        zeltcw1tqtaxkal,
        xnxngsssptdwpp9,
        px84pnizkjefrjv,
        vy5gpnlalyk3uqe,
        jjigcttpcc4kgvr,
        tzxgp1yojzignqn,
        stvng3tqieydrq8,
        o3mkx5fsxzdb6xj,
        jzvyxgaloz9lctw,
        bcgpjnrgynjfwwj,
        gsemwnocwhjnzgu,
        outepivfwv4h52h,
        u5etbohpnnrzvd3,
        tpte0wieuhcdjtz,
        unhyloxqfrfkir9,
        zvlcxvkrzkwvgte,
        ylkvg0eqotxebgj,
        tpauhhj6dedzqjl,
        ewql0vxpfwguadt,
        jqlkthnedwbftq5,
        pfgthxu5kjbkh1a,
        zrinkswmddy0twz,
        fozvwofjcsi4zdr,
        rvozwvlcnbzsx0o,
        p35gekrr3mehque,
        o2rov5ilpal6nlm,
        nwnhwu700fg9zlg,
        bsuj0hkyk4hqly3,
        qw6hfbvvexjlob9,
        alxvoubfnenggiw,
        mqdt6ge26az2qgu,
        txtjwoomqdz9p9z,
        shsjiahdushfagt,
        ni0aldylhpetftt,
        p55y589waasv9le,
        pmm8p2xpixf9q7p,
        zanjehlvft3xf7s,
        exsxjhtyoxwu7bq,
        p1u5djyua2ydz44,
        hkubhwyjceanir0,
        p7fbldpj3sqbprd,
        zkxwhtuekkmzapw,
        dwgqwywf9j7uyov,
        sdirpejs26hfs8e,
        robeb0bonfrw2on,
        wvz8tprvxo2kzfb,
        c9xscftbiahi2ct,
        fhlbzchdpsncka7,
        hce0exr8i1r4ee3,
        ekr6g3kxkzppfau,
        qv66d656ceuizxy,
        b0pfth6hzbomvlu,
        gh6e5vyhjjfy7af,
        atoncbipedbqopx,
        l2f70t318q5zt6j,
        f5xzwgaydxjans9,
        ssu7cnjtd3zsdtk,
        nzqpzizhd7fdce0,
        ovf9owerck9xduu,
        q432gspiybialjv,
        sodg7htwhflpyf1,
        k3kiijp2o9b5tpu,
        xasdcyo1lrdpdt2,
        fotsfs3bknv04k2 > String,
        v5w5do8ydmgobgc,
        b2uvlpqkeow9wfv,
        xyaodvfdznals7b,
        etyiz08lgvfq54q,
        uwzylfoypg5qlrm,
        x4yzsiwgq6trvin,
        pxf1zyg0kw5uwvz,
        mx55dlgfaeaw48o,
        rne3i6e1ubqd60k,
        jkrhrb6oin3t655,
        hr6f0ovnue4vns9,
        mtrpaer3idqubcd,
        wfaap4yqmxh74dl,
        yv1ewskjnlmhh08,
        iqfzmusiwzqtgvl,
        dg1jstxmav2f07r,
        zsf4i8jvkc22gxw,
        ixqpl5h6osulxnr,
        vavnpqqf2tbc02v,
        lyw29zr5bcdmrnl,
        srniyhbijfouuxc,
        fyuprxvjzsz7zlg
    ]);
    const LARGE_C: Seg/* <O> */ = Seg::new("example", "an example program").nest(&[
      Seg::new("ceW5VGwfWYYvYLK", "...").nest(&[
          Seg::new("Ojf1uclq5apmSwM", "..."),
          Seg::new("0QkusbzzsvAxaCg", "..."),
          Seg::new("CNW3qyguGykclun", "...").nest(&[
              Seg::new("nx4uTjCJVQKVGbY", "..."),
              Seg::new("ZC8rqomQ8XIlPKE", "..."),
              Seg::new("Nr6Ia12fGHFoSZd", "...").nest(&[
                  Seg::new("8iJG0s8IZNEyd0G", "..."),
                  Seg::new("hpGVauW8lfrabwc", "..."),
                  Seg::new("KOrmt4MhO1bKHnI", "..."),
                  Seg::new("XB6pfJwBWQNw7rM", "..."),
                  Seg::new("Zh6GH8GK28eZT2F", "..."),
                  Seg::new("ejvCsv0YiybxT9V", "..."),
                  Seg::new("RASvTDj67Vlud6L", "..."),
                  Seg::new("74haeRSRYKH4I2a", "..."),
                  Seg::new("HvpwGWTmFiQ0xLe", "..."),
                  Seg::new("Cd72emszDr2NOU7", "..."),
                  Seg::new("DEH8npzf9PAXOYg", "..."),
                  Seg::new("zvvcPdbmw0KXiIZ", "..."),
                  Seg::new("i6ik7dOzq9EUy3R", "..."),
              ]),
              Seg::new("rBY6OhfHy0QD1JO", "..."),
              Seg::new("thGKET7beOfY66j", "..."),
              Seg::new("AZp05xi7ok1GAdI", "...").nest(&[
                  Seg::new("rORmacHCNstiTvb", "..."),
                  Seg::new("7PS998WCYGfAUG5", "..."),
                  Seg::new("Rk1T4B9lE7AJonc", "..."),
                  Seg::new("6LjXIVkFBs1surc", "..."),
                  Seg::new("3fcm8FYKXGvKpEX", "...").nest(&[
                      Seg::new("9X6SfruJnj10uw9", "..."),
                      Seg::new("JJqprqpPacEe7bC", "..."),
                      Seg::new("dnqUXAcZDdWcaVG", "..."),
                      Seg::new("0UFqHiFiy4oamGd", "..."),
                      Seg::new("qgXnykW6YC31TSl", "..."),
                      Seg::new("n7mPtBTNG88uilL", "..."),
                      Seg::new("wbn7Am2B9s34Kam", "..."),
                      Seg::new("tbQnYaFsGVJwUcy", "..."),
                      Seg::new("V4tWTeRogoz3Trc", "..."),
                  ]),
                  Seg::new("HwVsi8xjSgorPAq", "..."),
                  Seg::new("klYlnsdA9eIFnSb", "..."),
                  Seg::new("wuOdI8EDVc8T1Ug", "..."),
                  Seg::new("g25NU7lRbmsjB03", "..."),
                  Seg::new("Cqby3r0nO57l5zS", "..."),
                  Seg::new("R99DQ7nNSIlAoCJ", "..."),
                  Seg::new("CyLLKqN8RstxpMb", "..."),
                  Seg::new("6sxcJLTVFIgPnlE", "..."),
              ]),
              Seg::new("ShJmy2WeuSNEYsa", "..."),
              Seg::new("pDBlHjdohenHNsX", "..."),
              Seg::new("qgvkpyR2N7WbdV0", "..."),
          ]),
          Seg::new("UfuPXBbGdavXZ1Q", "..."),
          Seg::new("WDQlWxEAnhvWpFz", "..."),
          Seg::new("hiXPX6Q4dB3BX9n", "..."),
          Seg::new("z0gAe9PXlXqpQfV", "..."),
          Seg::new("bc11CLBd7x1IYS4", "..."),
          Seg::new("JZ3ATwBBf29of0S", "..."),
          Seg::new("W9TdyzBgb629pyl", "..."),
      ]),
      Seg::new("XjYQFaKuUVXQEqs", "..."),
      Seg::new("UFWxl4yUWBq8nbJ", "..."),
      Seg::new("HQCdq6lnZhV7bdk", "...").action(|_| Ok(())),
      Seg::new("jNORDpDfjFWcuSq", "...").nest(&[
          Seg::new("vUl1ym3MO6xDYuq", "..."),
          Seg::new("8jBMJspIkYjhMUq", "..."),
          Seg::new("7oX5dxvM5Hc4elW", "..."),
          Seg::new("RoNvFg3NLohiXO0", "..."),
          Seg::new("bj3lrE1rAEtiJEY", "..."),
          Seg::new("XXdRXQvaoqqAcdV", "..."),
          Seg::new("53C0Fazoaag6YIH", "..."),
          Seg::new("1GJFcUw2ZgfLtvO", "..."),
          Seg::new("bpraMIKLXtRINqH", "..."),
          Seg::new("59gBEyZs1wTyyxk", "..."),
          Seg::new("WUBOLYKvDIifRVs", "..."),
          Seg::new("tE9FK0sWSAtEtBs", "..."),
          Seg::new("EgWtuaxYpXHbLKQ", "..."),
          Seg::new("Nr0GMJmJ8O41gpE", "..."),
          Seg::new("R9MuumXZIdY3cSf", "...").nest(&[Seg::new(
              "g4rW6Uz1wglHEOX",
              "...",
          )
          .nest(&[Seg::new("6wHGG1EGvEgs7p4", "...").nest(&[
              Seg::new("f7YKUV8iElaWysZ", "...").nest(&[Seg::new(
                  "cYHTZDar5J3Fr2a",
                  "...",
              )
              .nest(&[Seg::new("nnsC8YglQGgcQMK", "...")])]),
          ])])]),
          Seg::new("a5UQgE2HYK6EYME", "..."),
          Seg::new("ni7sgC5qeQjNSh5", "..."),
          Seg::new("hCZGL0vYEs53KRf", "..."),
      ]),
      Seg::new("PREZc7lWwQnPute", "..."),
      Seg::new("ROmgsRcBfuefTCB", "...").action(|_| Ok(())),
      Seg::new("CckrTLdVTlC6iop", "..."),
      Seg::new("Yii762E0GpV9Ev8", "..."),
      Seg::new("bnN1MkBa3WPeIjO", "...").action(|_| Ok(())),
      Seg::new("zDPaw4yaf5qrCDN", "...").action(|_| Ok(())),
      Seg::new("Yw8Yr3UjyKxyaT3", "..."),
      Seg::new("QF1XILBcUHnw3qq", "..."),
  ]);
    const LARGE_R: Router = router!(Large_O, LARGE_C);
    let large_args = vec![
        OsString::from_str("--pr6vjra6oaakimd").unwrap(),
        OsString::from_str("1").unwrap(),
        OsString::from_str("--lqlhwk41mtp4se0").unwrap(),
        OsString::from_str("11").unwrap(),
        OsString::from_str("--r9pkurgvexrchla").unwrap(),
        OsString::from_str("--value-arg").unwrap(),
        OsString::from_str("b").unwrap(),
        OsString::from_str("--den57vpd7mmggz9").unwrap(),
        OsString::from_str("--rm4jpdgggguvxnn").unwrap(),
        OsString::from_str("33").unwrap(),
        OsString::from_str("--pr6vjra6oaakimd").unwrap(),
        OsString::from_str("2").unwrap(),
        OsString::from_str("--d9thg7eanlkmmdv").unwrap(),
        OsString::from_str("4").unwrap(),
        OsString::from_str("--pr6vjra6oaakimd").unwrap(),
        OsString::from_str("3").unwrap(),
        OsString::from_str("--soozcjjoei7cq5o").unwrap(),
        OsString::from_str("--value-arg").unwrap(),
        OsString::from_str("b2").unwrap(),
        OsString::from_str("--fotsfs3bknv04k2").unwrap(),
        OsString::from_str("c").unwrap(),
    ];

    let mut group = c.benchmark_group("Routing");
    group.bench_function("Assign Small", |b| {
        let c = SMALL_R.parse(small_args.to_owned()).unwrap();
        b.iter(|| {
            let mut _args = AppArgs {
                number: c
                    .opt::<i32>(Small_O::Number)
                    .unwrap()
                    .and_then(|args| Some(args[0]))
                    .unwrap_or(0),
                opt_number: c
                    .opt::<u32>(Small_O::OptNumber)
                    .unwrap()
                    .and_then(|args| Some(args[0])),
                width: c
                    .opt::<u16>(Small_O::Width)
                    .unwrap()
                    .and_then(|args| Some(args[0]))
                    .unwrap_or(0),
                input: c
                    .operands()
                    .iter()
                    .map(|arg| PathBuf::from(arg))
                    .collect::<Vec<PathBuf>>(),
            };
        })
    });
    group.bench_function("Assign Large", |b| {
        let c = LARGE_R.parse(large_args.to_owned()).unwrap();
        b.iter(|| {
            let mut _args = AppArgs {
                number: c
                    .opt::<i32>(Large_O::pr6vjra6oaakimd)
                    .unwrap()
                    .and_then(|args| Some(args[0]))
                    .unwrap_or(0),
                opt_number: c
                    .opt::<u32>(Large_O::lqlhwk41mtp4se0)
                    .unwrap()
                    .and_then(|args| Some(args[0])),
                width: c
                    .opt::<u16>(Large_O::d9thg7eanlkmmdv)
                    .unwrap()
                    .and_then(|args| Some(args[0]))
                    .unwrap_or(0),
                input: c
                    .operands()
                    .iter()
                    .map(|arg| PathBuf::from(arg))
                    .collect::<Vec<PathBuf>>(),
            };
        })
    });
    group.bench_function("Parse Small", |b| {
        b.iter(|| SMALL_R.parse(small_args.to_owned()).unwrap())
    });
    group.bench_function("Parse Large", |b| {
        b.iter(|| LARGE_R.parse(large_args.to_owned()).unwrap())
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

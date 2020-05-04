use serde::{Serialize, Deserialize};
fn csv<T: serde::de::DeserializeOwned, P: AsRef<std::path::Path>>(path: P) -> Vec<T> {
    csv::ReaderBuilder::new()
        .from_path(path.as_ref())
        .unwrap_or_else(|e| panic!("couldn't read from file: {}", e))
        .into_deserialize()
        .collect::<Result<Vec<T>, _>>()
        .unwrap_or_else(|e| panic!("couldn't deserialize: {}", e))
}

fn main() {
    use serde_json::to_string_pretty as string;

    let mut args = std::env::args();
    args.next();
    let sub_cmd = args.next().expect("no sub command supplied");

    let mut path = std::env::current_dir().expect("no current dir");
    path.push(args.next().expect("no file supplied"));

    std::fs::write(
        &path.with_extension("json"),
        match sub_cmd.as_str() {
            "advancements" => string({
                #[derive(Deserialize)]
                struct Advancement {
                    xp: String,
                    title: String,
                    description: String,
                    plots: usize,
                    achiever_title: String,
                }

                #[derive(Serialize)]
                enum Kind {
                    Land { pieces: usize }
                }
                #[derive(Serialize)]
                struct Adv {
                    xp: String,
                    title: String,
                    description: String,
                    achiever_title: String,
                    kind: Kind
                }

                let mut total = 0;

                &csv::<Advancement, _>(&path)
                    .into_iter()
                    .map(|Advancement { xp, title, description, plots, achiever_title } | {
                        Adv {
                            xp,
                            title,
                            description,
                            achiever_title,
                            kind: Kind::Land {
                                pieces: {
                                    let r = plots - total;
                                    total += r;
                                    r
                                }
                            }
                        }
                    })
                    .collect::<Vec<Adv>>()
            }),
            "drops" => string({
                #[derive(Serialize)]
                enum Hatch {
                    Gp(usize, usize),
                    Item {
                        count: (usize, usize),
                        name: String
                    },
                    Rehatch,
                    Nothing,
                }
                #[derive(Deserialize)]
                struct SpawnRow {
                    chance: f32,
                    spawn: String,
                }

                &csv::<SpawnRow, _>(&path)
                    .into_iter()
                    .map(|SpawnRow { chance, spawn }| {
                        (
                            chance,
                            match spawn.as_str() {
                                "GP Haul" => Hatch::Gp(100, 200),
                                "Multidrop" | "Double Drop" => Hatch::Rehatch,
                                "Nothing Happens" => Hatch::Nothing,
                                _ => Hatch::Item { count: (1, 1), name: spawn }
                            }
                        )
                    })
                    .collect::<Vec<_>>()
            }),
            _ => panic!("unknown subcommand: {}", sub_cmd)
        }.expect("couldn't serialize json")
    ).expect("couldn't write");
}

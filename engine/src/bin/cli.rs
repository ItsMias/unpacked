//! `unpacked-cli <package.zip> [--no-analytics] [--no-tns] [--debug-games]`
//!
//! Runs the engine natively and prints the facts JSON. Used to validate against a real package.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::time::Instant;

use engine::{Engine, Entry, wanted};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = args.first().ok_or("usage: unpacked-cli <package.zip> [--no-analytics] [--no-tns] [--debug-games]")?;

    let started = Instant::now();
    let mut zip = zip::ZipArchive::new(BufReader::new(File::open(path)?))?;
    let mut engine = Engine::new();

    let mut singles = vec![];
    let mut channels: HashMap<String, (Option<Vec<u8>>, Option<Vec<u8>>)> = HashMap::new();
    let (mut events, mut fallback, mut analytics) = (vec![], vec![], vec![]);
    for i in 0..zip.len() {
        let name = zip.name_for_index(i).unwrap_or_default().to_owned();
        match wanted(&name) {
            Some(Entry::Events) => events.push(i),
            Some(Entry::EventsFallback) => fallback.push(i),
            Some(Entry::Analytics) => analytics.push(i),
            Some(k @ (Entry::ChannelMeta | Entry::ChannelMessages)) => {
                let folder = name.rsplit_once('/').unwrap().0.to_owned();
                let mut buf = vec![];
                zip.by_index(i)?.read_to_end(&mut buf)?;
                let slot = channels.entry(folder).or_default();
                if k == Entry::ChannelMeta { slot.0 = Some(buf) } else { slot.1 = Some(buf) }
            }
            Some(k) => singles.push((i, k, name)),
            None => {}
        }
    }
    // The user file first, so the rest can refer to it.
    singles.sort_by_key(|(_, k, _)| *k != Entry::User);
    for (i, kind, name) in &singles {
        let mut buf = vec![];
        zip.by_index(*i)?.read_to_end(&mut buf)?;
        engine.feed_file(*kind, &buf)?;
        eprintln!("read  {name}");
    }
    let n_channels = channels.len();
    for (folder, pair) in channels {
        if let (Some(c), Some(m)) = pair {
            if let Err(e) = engine.feed_channel(&c, &m) {
                eprintln!("skip  {folder}: {e}");
            }
        }
    }
    eprintln!("read  {n_channels} message folders   ({:.1}s)", started.elapsed().as_secs_f64());

    let mut stream = |engine: &mut Engine, idx: &[usize], is_analytics: bool| -> Result<(), Box<dyn std::error::Error>> {
        for &i in idx {
            let mut f = zip.by_index(i)?;
            let t = Instant::now();
            let name = f.name().to_owned();
            let mut buf = vec![0u8; 1 << 20];
            loop {
                let n = f.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                if is_analytics { engine.feed_analytics(&buf[..n]) } else { engine.feed_events(&buf[..n]) }
            }
            let n = engine.finish_events();
            eprintln!("read  {name}   {n} events ({:.1}s)", t.elapsed().as_secs_f64());
        }
        Ok(())
    };
    stream(&mut engine, &events, false)?;
    let mut tns = None;
    if !engine.has_voice() && !fallback.is_empty() {
        eprintln!("no voice events in reporting, trying tns");
        stream(&mut engine, &fallback, false)?;
    } else if !fallback.is_empty() && !args.iter().any(|a| a == "--no-tns") {
        // What only the trust & safety log has, from a second engine, like the page's own worker.
        let mut extra = Engine::new();
        stream(&mut extra, &fallback, false)?;
        tns = extra.finish().tns;
    }
    if !args.iter().any(|a| a == "--no-analytics") {
        stream(&mut engine, &analytics, true)?;
    }

    if args.iter().any(|a| a == "--debug-games") {
        for g in engine.all_games() {
            eprintln!("game  {:>4}  {}", g.sessions, g.name);
        }
    }
    let mut facts = engine.finish();
    if tns.is_some() {
        facts.tns = tns;
    }
    println!("{}", serde_json::to_string(&facts)?);
    eprintln!("total {:.1}s", started.elapsed().as_secs_f64());
    Ok(())
}

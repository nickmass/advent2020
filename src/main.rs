use ahash::AHashMap as HashMap;

use std::io::Write;
use std::time::Duration;

use advent::solutions::{self, Solution, SolutionCollection};
pub use advent::Input;

mod downloader;
mod profiler;

use downloader::InputDownloader;
pub use profiler::{Metrics, Profiler};

const DEFAULT_EVENT: u32 = 2024;

enum EventSelection {
    Specific(u32),
    All,
}

fn main() {
    let events: HashMap<_, _> = [
        (2019, solutions::days_2019()),
        (2020, solutions::days_2020()),
        (2021, solutions::days_2021()),
        (2022, solutions::days_2022()),
        (2023, solutions::days_2023()),
        (2024, solutions::days_2024()),
    ]
    .into_iter()
    .collect();

    let mut args: Vec<_> = std::env::args().skip(1).collect();

    let mut submission = false;
    let mut details = false;
    let mut print_input = false;

    for arg in args.iter() {
        match arg.as_str() {
            "--submit" | "-s" => submission = true,
            "--details" | "-d" => details = true,
            "--input" | "-i" => print_input = true,
            "--help" | "-h" => {
                usage();
                return;
            }
            _ => (),
        }
    }

    args.retain(|arg| !arg.starts_with("-"));

    let event = if let Some(arg) = args.first() {
        if arg.to_lowercase() == "all" {
            EventSelection::All
        } else {
            match arg.parse() {
                Ok(event) => {
                    if events.contains_key(&event) {
                        EventSelection::Specific(event)
                    } else {
                        eprintln!("event '{}' not found", event);
                        std::process::exit(1)
                    }
                }
                Err(_err) => {
                    eprintln!("invalid event '{}'", arg);
                    std::process::exit(1)
                }
            }
        }
    } else {
        EventSelection::Specific(DEFAULT_EVENT)
    };

    let day_filter: Vec<_> = args
        .into_iter()
        .skip(1)
        .filter_map(|arg| arg.parse().ok())
        .collect();

    let downloader = InputDownloader::new();
    let profiler = Profiler::new();

    let mut context = Context {
        downloader,
        profiler,
        details,
        submission,
        print_input,
    };

    match event {
        EventSelection::Specific(event) => {
            if let Some(days) = events.get(&event) {
                run_event(&mut context, event, days, &day_filter);
            } else {
                eprintln!("event '{}' not configured", event);
                std::process::exit(1)
            }
        }
        EventSelection::All => {
            let mut events: Vec<_> = events.into_iter().collect();
            events.sort_by_key(|e| e.0);

            let mut overall_duration = Duration::ZERO;

            for event in &events {
                overall_duration += run_event(&mut context, event.0, &event.1, &day_filter);
                println!();
                println!();
            }

            println!("Overall duration{:>26}", profiler::Time(overall_duration))
        }
    }
}

struct Context {
    downloader: InputDownloader,
    profiler: Profiler,
    details: bool,
    submission: bool,
    print_input: bool,
}

fn run_event(
    ctx: &mut Context,
    event: u32,
    days: &SolutionCollection,
    day_filter: &[u32],
) -> Duration {
    println!("Advent of Code - {}", event);
    println!();

    let mut total_duration = Duration::ZERO;

    for day in days
        .solutions()
        .filter(|s| day_filter.is_empty() || day_filter.contains(&s.day))
    {
        total_duration += run_day(ctx, event, day)
    }
    println!("Total duration{:>28}", profiler::Time(total_duration));

    total_duration
}

fn run_day(ctx: &mut Context, event: u32, day: &Solution) -> Duration {
    match ctx.downloader.download_input_if_absent(event, day.day) {
        Ok(input) => {
            if ctx.print_input {
                println!();
                println!("{input}");
                println!();
            }

            ctx.profiler.start();
            let part_one = (day.part_one)(&input);
            let part_one_metrics = ctx.profiler.stop();
            let part_one = part_one.to_string();

            print_line(ctx, event, day.day, 1, &part_one, &part_one_metrics);

            ctx.profiler.start();
            let part_two = (day.part_two)(&input);
            let part_two_metrics = ctx.profiler.stop();
            let part_two = part_two.to_string();

            print_line(ctx, event, day.day, 2, &part_two, &part_two_metrics);

            if ctx.submission {
                submit_day_part(ctx, event, day, 1, &part_one);
                submit_day_part(ctx, event, day, 2, &part_two);
            }

            part_one_metrics.duration + part_two_metrics.duration
        }
        Err(error) => {
            eprintln!(
                "unable to get input for '{}' day '{}'. {:?}",
                event, day.day, error
            );
            std::process::exit(1)
        }
    }
}

fn submit_day_part(ctx: &Context, event: u32, day: &Solution, part: u32, answer: &str) {
    print!("Submit part {}? [ycN] ", part);
    std::io::stdout().flush().unwrap();
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();

    let entry = buffer.trim().to_ascii_lowercase();
    let answer = if entry == "c" {
        print!("Enter custom submission: ");
        std::io::stdout().flush().unwrap();
        buffer.clear();
        std::io::stdin().read_line(&mut buffer).unwrap();

        Some(buffer.trim())
    } else if entry == "y" {
        Some(answer)
    } else {
        None
    };

    if let Some(answer) = answer {
        let res = ctx.downloader.submit_answer(answer, event, day.day, part);

        match res {
            Ok(true) => println!("Correct"),
            Ok(false) => println!("Incorrect"),
            Err(err) => {
                eprintln!(
                    "unable to submit answer for '{}' day '{}' part '{}'. {:?}",
                    event, day.day, part, err
                );
            }
        }
    }
}

fn print_line<S: AsRef<str>>(
    ctx: &Context,
    _event: u32,
    day: u32,
    part: u32,
    answer: S,
    metrics: &Metrics,
) {
    let answer = answer.as_ref();
    if answer.len() <= 25 {
        println!(
            "{:>2}-{}:{:>25}{}",
            day,
            part,
            answer,
            metrics.display(ctx.details)
        );
    } else {
        println!(
            "{:>2}-{}:{:>25}{}",
            day,
            part,
            "",
            metrics.display(ctx.details)
        );
        println!("{}", answer);
    }
}

fn usage() {
    let usage = "Usage: advent-of-code [OPTIONS] [EVENT] [DAY]...

Arguments:

	[EVENT]
		Limit to solutions from a speific year, defaults to most recent year, 'all' will execute all events.

	[DAY]...
		Only execute specified days within the choosen event.

Options:

	--submit	-s
		Ask to submit answer after each solution. Requires '.session-key' file containing an Advent of Code authentication cookie value in the working directory.

	--details	-d
		Display additional performance metrics.
		
	--input    -i
	   Print the input for each day

	--help	-h
		Display this message.

Examples:

	advent-of-code 2021
		Execute all solutions from 2021.

	advent-of-code -s 2023 22
		Execute parts 1 and 2 from day 22 of 2023, prompt to submit the solution after each part.

	advent-of-code -d all
		Execute all solutions with detailed metrics.
";
    println!("{}", usage);
}

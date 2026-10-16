//! Technical jokes and dynamic greetings for PDFCatalyst.

use std::sync::atomic::{AtomicUsize, Ordering};

/// A curated list of witty, clean technical and programming jokes (no emojis).
pub const TECH_JOKES: &[&str] = &[
    "Why do programmers prefer dark mode? Because light attracts bugs.",
    "There are 10 types of people in the world: those who understand binary, and those who don't.",
    "A SQL query walks into a bar, walks up to two tables and asks: 'Can I join you?'",
    "Why do Rust developers sleep so soundly? Because their memory is safe.",
    "To understand recursion, you must first understand recursion.",
    "Why did the developer go broke? Because they used up all their cache.",
    "Hardware is the part of a computer you can kick; software is the part you can only curse at.",
    "There are two hard things in Computer Science: cache invalidation, naming things, and off-by-one errors.",
    "An optimist says the glass is half full. A pessimist says it's half empty. A programmer says it's twice as large as necessary.",
    "Why did the PDF cross the road? To preserve its layout on the other side.",
    "!false — It's funny because it's true.",
    "A programmer's spouse asks: 'Go to the store and get a loaf of bread. If they have eggs, get a dozen.' They return with 12 loaves of bread.",
    "Algorithm: A word used by programmers when they don't want to explain what they did.",
    "Rust compiler: 'I found an error in your code, and also here is how to fix your life.'",
    "Why was the JavaScript developer sad? Because they didn't Node how to Express themselves.",
    "Git commit -m 'Fixed it for real this time, I swear.'",
    "Documentation is like love: even when it's bad, it's better than nothing.",
    "Real programmers count from 0.",
    "Debugging: Being the detective in a crime movie where you are also the murderer.",
    "There is no place like 127.0.0.1.",
    "Why do Java developers wear glasses? Because they can't C#.",
    "A QA engineer walks into a bar. Orders a beer. Orders 0 beers. Orders 999999999 beers. Orders a lizard. Orders -1 beers.",
    "Chuck Norris can divide by zero and dereference a null pointer.",
    "Ship it, it worked on my machine.",
    "A good programmer looks both ways before crossing a one-way street.",
    "Keyboard not found. Press F1 to continue.",
    "A programmer had a problem. They decided to use threads. Now have two they problems.",
];

static JOKE_INDEX: AtomicUsize = AtomicUsize::new(0);

/// Returns the current joke.
pub fn current_joke() -> &'static str {
    let idx = JOKE_INDEX.load(Ordering::Relaxed) % TECH_JOKES.len();
    TECH_JOKES[idx]
}

/// Advances to the next joke and returns it.
pub fn next_joke() -> &'static str {
    let idx = JOKE_INDEX.fetch_add(1, Ordering::Relaxed) + 1;
    TECH_JOKES[idx % TECH_JOKES.len()]
}

/// Dynamic greetings based on system time and technical flavor (no emojis).
pub fn dynamic_greeting() -> &'static str {
    let hour = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => ((d.as_secs() / 3600 + 5) % 24) as u32,
        Err(_) => 12,
    };

    match hour {
        5..12 => "Good morning, Engineer. Coffee brewed, zero memory leaks guaranteed.",
        12..17 => "Good afternoon. High-throughput vector processing active.",
        17..22 => "Good evening. Smooth vector rendering for your evening sprint.",
        _ => "Late-night session active. Rust memory safety guarantees remain unwavering.",
    }
}

/// A curated list of inspiring quotes on design, engineering, and craftsmanship.
pub const INSPIRING_QUOTES: &[(&str, &str)] = &[
    ("Simplicity is prerequisite for reliability.", "Edsger W. Dijkstra"),
    ("Perfection is achieved not when there is nothing more to add, but when there is nothing left to take away.", "Antoine de Saint-Exupéry"),
    ("Make it simple, but significant.", "Don Draper"),
    ("The details are not the details. They make the design.", "Charles Eames"),
    ("Craftsmanship is not an accident; it is the deliberate pursuit of mastery.", "Anonymous"),
    ("Simplicity is the soul of efficiency.", "Austin Freeman"),
    ("First, solve the problem. Then, write the code.", "John Johnson"),
    ("Quality is not an act, it is a habit.", "Aristotle"),
    ("Good code is its own best documentation.", "Steve McConnell"),
    ("Design is not just what it looks like and feels like. Design is how it works.", "Steve Jobs"),
    ("Programs must be written for people to read, and only incidentally for machines to execute.", "Harold Abelson"),
    ("Focus is a practice of saying no to a hundred other good ideas.", "Steve Jobs"),
];

static QUOTE_INDEX: AtomicUsize = AtomicUsize::new(0);

/// Returns the current inspiring quote and its author.
pub fn current_quote() -> (&'static str, &'static str) {
    let idx = QUOTE_INDEX.load(Ordering::Relaxed) % INSPIRING_QUOTES.len();
    INSPIRING_QUOTES[idx]
}

/// Advances to the next quote and returns it.
pub fn next_quote() -> (&'static str, &'static str) {
    let idx = QUOTE_INDEX.fetch_add(1, Ordering::Relaxed) + 1;
    INSPIRING_QUOTES[idx % INSPIRING_QUOTES.len()]
}

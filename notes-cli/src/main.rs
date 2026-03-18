mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "notes", about = "Typst-based note-taking CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new vault
    Init {
        #[arg(default_value = ".")]
        path: String,
    },
    /// Create a new note (supports hierarchy: "parent/child/note")
    New {
        title: String,
        #[arg(long, default_value = "note")]
        r#type: String,
    },
    /// Delete a note by id
    Delete {
        id: String,
    },
    /// Rename a note (updates all references)
    Rename {
        old_id: String,
        new_id: String,
    },
    /// Rebuild the notes index
    Index,
    /// Sync CSV with filesystem + reindex
    Sync,
    /// Compile a note to HTML or PDF (accepts id or file path)
    Compile {
        note: String,
        #[arg(long, default_value = "html")]
        format: String,
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Search notes
    Search {
        query: String,
        #[arg(long)]
        r#type: Option<String>,
    },
    /// Show backlinks for a note
    Backlinks {
        id: String,
    },
    /// List all notes
    List {
        #[arg(long)]
        r#type: Option<String>,
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// Watch a note and recompile on changes (accepts id or file path)
    Watch {
        note: String,
        #[arg(long, default_value = "html")]
        format: String,
    },
    /// Compile vault graph
    Graph {
        #[arg(long, default_value = "html")]
        format: String,
        #[arg(short, long)]
        output: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { path } => commands::init(&path),
        Commands::New { title, r#type } => commands::new_note(&title, &r#type),
        Commands::Delete { id } => commands::delete(&id),
        Commands::Rename { old_id, new_id } => commands::rename(&old_id, &new_id),
        Commands::Index => commands::index(),
        Commands::Sync => commands::sync(),
        Commands::Compile { note, format, output } => {
            commands::compile(&note, &format, output.as_deref())
        }
        Commands::Watch { note, format } => commands::watch(&note, &format),
        Commands::Search { query, r#type } => commands::search(&query, r#type.as_deref()),
        Commands::Backlinks { id } => commands::backlinks(&id),
        Commands::List { r#type, format } => commands::list(r#type.as_deref(), &format),
        Commands::Graph { format, output } => commands::graph(&format, output.as_deref()),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

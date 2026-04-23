mod common;

use common::{assert_success, read, remove_repo, run_init, temp_repo, write};

const EXPECTED_CONFIG: &str = "version = 1\n\n[foundation]\npinned = [\"AGENTS.md\", \"CLAUDE.md\"]\n\n[languages]\nenabled = [\"python\", \"rust\", \"javascript\", \"typescript\", \"go\"]\n";

const EXPECTED_AGENTS: &str = "<!-- frugal:managed:start -->\n# frugal\n\n1. Run `fgl status` before starting a task to see current prefix/active ratio.\n2. Run `fgl pack <paths...>` instead of reading many source files directly when exploring.\n3. Treat Foundation slab as read-only cached context. Do not re-read pinned files raw unless you need to edit them.\n4. Read a file raw only when you need exact body content or plan to write to it.\n5. Prefer `fgl pack <active-file> > CONTEXT.md` when preparing context for an external model.\n<!-- frugal:managed:end -->\n";

const EXPECTED_CLAUDE: &str = "<!-- frugal:managed:start -->\n# frugal\n\n1. Run `fgl status` before starting a task to see current prefix/active ratio.\n2. Run `fgl pack <paths...>` instead of reading many source files directly when exploring.\n3. Treat Foundation slab as read-only cached context. Do not re-read pinned files raw unless you need to edit them.\n4. Read a file raw only when you need exact body content or plan to write to it.\n5. Prefer `fgl pack <active-file> > CONTEXT.md` when preparing context for an external model.\n<!-- frugal:managed:end -->\n";

#[test]
fn init_creates_config_agents_claude_in_empty_repo() {
    let repo = temp_repo();

    let output = run_init(&repo);

    assert_success(&output);
    assert_eq!(read(&repo, ".fgl/config.toml"), EXPECTED_CONFIG);
    assert_eq!(read(&repo, "AGENTS.md"), EXPECTED_AGENTS);
    assert_eq!(read(&repo, "CLAUDE.md"), EXPECTED_CLAUDE);

    remove_repo(&repo);
}

#[test]
fn init_second_run_is_byte_identical() {
    let repo = temp_repo();

    assert_success(&run_init(&repo));
    let first_config = read(&repo, ".fgl/config.toml");
    let first_agents = read(&repo, "AGENTS.md");
    let first_claude = read(&repo, "CLAUDE.md");

    assert_success(&run_init(&repo));

    assert_eq!(read(&repo, ".fgl/config.toml"), first_config);
    assert_eq!(read(&repo, "AGENTS.md"), first_agents);
    assert_eq!(read(&repo, "CLAUDE.md"), first_claude);

    remove_repo(&repo);
}

#[test]
fn init_preserves_user_text_outside_managed_markers() {
    let repo = temp_repo();
    write(
        &repo,
        "AGENTS.md",
        "intro line\n\n<!-- frugal:managed:start -->\nold agents\n<!-- frugal:managed:end -->\n\noutro line\n",
    );
    write(
        &repo,
        "CLAUDE.md",
        "claude intro\n\n<!-- frugal:managed:start -->\nold claude\n<!-- frugal:managed:end -->\n\nclaude outro\n",
    );

    assert_success(&run_init(&repo));

    assert_eq!(
        read(&repo, "AGENTS.md"),
        "intro line\n\n<!-- frugal:managed:start -->\n# frugal\n\n1. Run `fgl status` before starting a task to see current prefix/active ratio.\n2. Run `fgl pack <paths...>` instead of reading many source files directly when exploring.\n3. Treat Foundation slab as read-only cached context. Do not re-read pinned files raw unless you need to edit them.\n4. Read a file raw only when you need exact body content or plan to write to it.\n5. Prefer `fgl pack <active-file> > CONTEXT.md` when preparing context for an external model.\n<!-- frugal:managed:end -->\n\noutro line\n"
    );
    assert_eq!(
        read(&repo, "CLAUDE.md"),
        "claude intro\n\n<!-- frugal:managed:start -->\n# frugal\n\n1. Run `fgl status` before starting a task to see current prefix/active ratio.\n2. Run `fgl pack <paths...>` instead of reading many source files directly when exploring.\n3. Treat Foundation slab as read-only cached context. Do not re-read pinned files raw unless you need to edit them.\n4. Read a file raw only when you need exact body content or plan to write to it.\n5. Prefer `fgl pack <active-file> > CONTEXT.md` when preparing context for an external model.\n<!-- frugal:managed:end -->\n\nclaude outro\n"
    );

    remove_repo(&repo);
}

#[test]
fn init_updates_existing_managed_block_only() {
    let repo = temp_repo();
    write(
        &repo,
        "AGENTS.md",
        "before\n\n<!-- frugal:managed:start -->\nstale\n<!-- frugal:managed:end -->\n\nafter\n",
    );
    write(
        &repo,
        "CLAUDE.md",
        "before claude\n\n<!-- frugal:managed:start -->\nstale\n<!-- frugal:managed:end -->\n\nafter claude\n",
    );
    write(
        &repo,
        ".fgl/config.toml",
        "version = 1\n\n[foundation]\npinned = [\"custom.md\"]\n\n[languages]\nenabled = [\"python\"]\n",
    );

    assert_success(&run_init(&repo));

    assert_eq!(
        read(&repo, ".fgl/config.toml"),
        "version = 1\n\n[foundation]\npinned = [\"custom.md\"]\n\n[languages]\nenabled = [\"python\"]\n"
    );
    assert_eq!(
        read(&repo, "AGENTS.md"),
        "before\n\n<!-- frugal:managed:start -->\n# frugal\n\n1. Run `fgl status` before starting a task to see current prefix/active ratio.\n2. Run `fgl pack <paths...>` instead of reading many source files directly when exploring.\n3. Treat Foundation slab as read-only cached context. Do not re-read pinned files raw unless you need to edit them.\n4. Read a file raw only when you need exact body content or plan to write to it.\n5. Prefer `fgl pack <active-file> > CONTEXT.md` when preparing context for an external model.\n<!-- frugal:managed:end -->\n\nafter\n"
    );
    assert_eq!(
        read(&repo, "CLAUDE.md"),
        "before claude\n\n<!-- frugal:managed:start -->\n# frugal\n\n1. Run `fgl status` before starting a task to see current prefix/active ratio.\n2. Run `fgl pack <paths...>` instead of reading many source files directly when exploring.\n3. Treat Foundation slab as read-only cached context. Do not re-read pinned files raw unless you need to edit them.\n4. Read a file raw only when you need exact body content or plan to write to it.\n5. Prefer `fgl pack <active-file> > CONTEXT.md` when preparing context for an external model.\n<!-- frugal:managed:end -->\n\nafter claude\n"
    );

    remove_repo(&repo);
}

#[test]
fn init_preserves_existing_valid_toml_config() {
    let repo = temp_repo();
    let existing = "# repo-specific config\nversion = 1\n\n[foundation]\npinned = [\n  'custom.md',\n  \"notes.md\",\n]\n\n[languages]\nenabled = [\"python\", \"rust\"]\n";
    write(&repo, ".fgl/config.toml", existing);

    assert_success(&run_init(&repo));

    assert_eq!(read(&repo, ".fgl/config.toml"), existing);

    remove_repo(&repo);
}

<!-- frugal:managed:start -->
# frugal

1. Run `fgl status` before starting a task to see current prefix/active ratio.
2. Run `fgl pack <paths...>` instead of reading many source files directly when exploring.
3. Treat Foundation slab as read-only cached context. Do not re-read pinned files raw unless you need to edit them.
4. Read a file raw only when you need exact body content or plan to write to it.
5. Prefer `fgl pack <active-file> > CONTEXT.md` when preparing context for an external model.
<!-- frugal:managed:end -->

package grouped

const limit = computeLimit()

var (
	// Counter docs.
	counter int
	names   = loadNames()
)

type (
	// Alias docs.
	Alias = map[string]int

	// Worker docs.
	Worker interface {
		Work() error
	}
)

// Run docs.
func Run() error {
	return nil
}

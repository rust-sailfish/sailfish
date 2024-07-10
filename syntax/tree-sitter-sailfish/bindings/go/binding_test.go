package tree_sitter_sailfish_test

import (
	"testing"

	tree_sitter "github.com/smacker/go-tree-sitter"
	"github.com/tree-sitter/tree-sitter-sailfish"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_sailfish.Language())
	if language == nil {
		t.Errorf("Error loading Sailfish grammar")
	}
}
